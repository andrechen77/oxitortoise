use std::mem::offset_of;

use derive_more::derive::Display;
use lir::smallvec::{SmallVec, smallvec};
use slotmap::SlotMap;

use crate::{
    exec::CanonExecutionContext,
    mir::{EffectfulNode, LocalDeclaration, LocalId, NodeId, build_lir::LirInsnBuilder, node},
    sim::{
        patch::{self, PatchVarDesc},
        turtle::{self, TurtleVarDesc, calc_turtle_var_offsets},
        value::NetlogoInternalType,
    },
    workspace::Workspace,
};

#[derive(Debug, Display)]
#[display("GetTurtleVar {var:?}")]
pub struct GetTurtleVar {
    /// The execution context to use.
    pub context: NodeId,
    /// The turtle whose variable is being gotten.
    pub turtle: NodeId,
    /// The variable to get.
    pub var: TurtleVarDesc,
}

impl EffectfulNode for GetTurtleVar {
    fn has_side_effects(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.turtle]
    }

    fn output_type(
        &self,
        workspace: &Workspace,
        _nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<NetlogoInternalType> {
        Some(turtle::turtle_var_type(&workspace.world.turtles.schema(), self.var))
    }

    fn lowering_expand(
        &self,
        my_node_id: NodeId,
        workspace: &Workspace,
        nodes: &mut SlotMap<NodeId, Box<dyn EffectfulNode>>,
    ) -> bool {
        let (data_row, field_offset) =
            context_to_turtle_data(nodes, workspace, self.context, self.turtle, self.var);

        // create a node to get the field
        let field = Box::new(node::MemLoad {
            ptr: data_row,
            offset: field_offset,
            ty: turtle::turtle_var_type(&workspace.world.turtles.schema(), self.var),
        });
        nodes[my_node_id] = field;
        true
    }
}

#[derive(Debug, Display)]
#[display("SetTurtleVar {var:?}")]
pub struct SetTurtleVar {
    /// The execution context to use.
    pub context: NodeId,
    /// The turtle whose variable is being set.
    pub turtle: NodeId,
    /// The variable to set.
    pub var: TurtleVarDesc,
    /// The value to set the variable to.
    pub value: NodeId,
}

impl EffectfulNode for SetTurtleVar {
    fn has_side_effects(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.turtle, self.value]
    }

    fn output_type(
        &self,
        _workspace: &Workspace,
        _nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<NetlogoInternalType> {
        None
    }

    fn lowering_expand(
        &self,
        my_node_id: NodeId,
        workspace: &Workspace,
        nodes: &mut SlotMap<NodeId, Box<dyn EffectfulNode>>,
    ) -> bool {
        let (data_row, field_offset) =
            context_to_turtle_data(nodes, workspace, self.context, self.turtle, self.var);

        // create a node to set the field
        let field =
            Box::new(node::MemStore { ptr: data_row, offset: field_offset, value: self.value });
        nodes[my_node_id] = field;
        true
    }
}

/// Helper function to derive a pointer to the data row of a turtle's variable.
/// Returns the NodeId of the node that outputs the pointer to data row, as well
/// as the byte offset of the field within the data row. This is used by both
/// loads and stores.
fn context_to_turtle_data(
    nodes: &mut SlotMap<NodeId, Box<dyn EffectfulNode>>,
    workspace: &Workspace,
    context: NodeId,
    turtle_id: NodeId,
    var: TurtleVarDesc,
) -> (NodeId, usize) {
    let (buffer_offset, stride, field_offset) =
        calc_turtle_var_offsets(&workspace.world.turtles, var);

    // insert a node that gets the workspace pointer
    let workspace_ptr = nodes.insert(Box::new(node::MemLoad {
        ptr: context,
        offset: offset_of!(CanonExecutionContext, workspace),
        ty: NetlogoInternalType::UNTYPED_PTR,
    }));

    // insert a node that gets the row buffer
    let row_buffer = nodes.insert(Box::new(node::MemLoad {
        ptr: workspace_ptr,
        offset: offset_of!(Workspace, world) + buffer_offset,
        ty: NetlogoInternalType::UNTYPED_PTR,
    }));

    // insert a node that gets the agent index
    let turtle_idx = nodes.insert(Box::new(node::TurtleIdToIndex { turtle_id }));

    // insert a node that gets the right data row
    let data_row =
        nodes.insert(Box::new(node::DeriveElement { ptr: row_buffer, index: turtle_idx, stride }));

    (data_row, field_offset)
}

#[derive(Debug, Display)]
#[display("TurtleIdToIndex")]
pub struct TurtleIdToIndex {
    /// The turtle id to convert.
    pub turtle_id: NodeId,
}

impl EffectfulNode for TurtleIdToIndex {
    fn has_side_effects(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.turtle_id]
    }

    fn output_type(
        &self,
        _workspace: &Workspace,
        _nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<NetlogoInternalType> {
        Some(NetlogoInternalType::AGENT_INDEX)
    }

    fn write_lir_execution(
        &self,
        my_node_id: NodeId,
        nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), ()> {
        let &[turtle_id] = lir_builder.get_node_results(nodes, self.turtle_id) else {
            panic!("expected node outputting turtle id to be a single LIR value");
        };
        let pc = lir_builder.push_lir_insn(lir::InsnKind::UnaryOp {
            op: lir::UnaryOpcode::I64ToI32,
            operand: turtle_id,
        });
        lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(pc, 0)]);
        Ok(())
    }
}

#[derive(Debug, Display)]
#[display("GetPatchVar {var:?}")]
pub struct GetPatchVar {
    /// The execution context to use.
    pub context: NodeId,
    /// The patch whose variable is being gotten.
    pub patch: NodeId,
    /// The variable to get.
    pub var: PatchVarDesc,
}

impl EffectfulNode for GetPatchVar {
    fn has_side_effects(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.patch]
    }

    fn output_type(
        &self,
        workspace: &Workspace,
        _nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<NetlogoInternalType> {
        Some(patch::patch_var_type(&workspace.world.patches.schema(), self.var))
    }

    fn lowering_expand(
        &self,
        my_node_id: NodeId,
        workspace: &Workspace,
        nodes: &mut SlotMap<NodeId, Box<dyn EffectfulNode>>,
    ) -> bool {
        let (data_row, field_offset) =
            context_to_patch_data(nodes, workspace, self.context, self.patch, self.var);

        // create a node to get the field
        let field = Box::new(node::MemLoad {
            ptr: data_row,
            offset: field_offset,
            ty: patch::patch_var_type(&workspace.world.patches.schema(), self.var),
        });
        nodes[my_node_id] = field;
        true
    }
}

#[derive(Debug, Display)]
#[display("SetPatchVar {var:?}")]
pub struct SetPatchVar {
    /// The execution context to use.
    pub context: NodeId,
    /// The patch whose variable is being set.
    pub patch: NodeId,
    /// The variable to set.
    pub var: PatchVarDesc,
    /// The value to set the variable to.
    pub value: NodeId,
}

impl EffectfulNode for SetPatchVar {
    fn has_side_effects(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.patch, self.value]
    }

    fn output_type(
        &self,
        _workspace: &Workspace,
        _nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<NetlogoInternalType> {
        None
    }

    fn lowering_expand(
        &self,
        my_node_id: NodeId,
        workspace: &Workspace,
        nodes: &mut SlotMap<NodeId, Box<dyn EffectfulNode>>,
    ) -> bool {
        let (data_row, field_offset) =
            context_to_patch_data(nodes, workspace, self.context, self.patch, self.var);

        // create a node to set the field
        let field =
            Box::new(node::MemStore { ptr: data_row, offset: field_offset, value: self.value });
        nodes[my_node_id] = field;
        true
    }
}

/// Helper function to derive a pointer to the data row of a patch's variable. Returns the
/// the NodeId of the node that outputs the pointer to data row, as well as the byte
/// offset of the field within the data row. This is used by both loads and stores.
fn context_to_patch_data(
    nodes: &mut SlotMap<NodeId, Box<dyn EffectfulNode>>,
    workspace: &Workspace,
    context: NodeId,
    patch_id: NodeId,
    var: PatchVarDesc,
) -> (NodeId, usize) {
    let (buffer_offset, stride, field_offset) =
        patch::calc_patch_var_offsets(&workspace.world.patches, var);

    // insert a node that gets the workspace pointer
    let workspace_ptr = nodes.insert(Box::new(node::MemLoad {
        ptr: context,
        offset: offset_of!(CanonExecutionContext, workspace),
        ty: NetlogoInternalType::UNTYPED_PTR,
    }));

    // insert a node that gets the row buffer
    let row_buffer = nodes.insert(Box::new(node::MemLoad {
        ptr: workspace_ptr,
        offset: offset_of!(Workspace, world) + buffer_offset,
        ty: NetlogoInternalType::UNTYPED_PTR,
    }));

    // insert a node that gets the right data row
    let data_row =
        nodes.insert(Box::new(node::DeriveElement { ptr: row_buffer, index: patch_id, stride }));

    (data_row, field_offset)
}

/// A node for getting an patch variable when the type of the agent is unknown.
#[derive(Debug, Display)]
#[display("GetPatchVarAsTurtleOrPatch {var:?}")]
pub struct GetPatchVarAsTurtleOrPatch {
    /// The execution context to use.
    pub context: NodeId,
    /// The patch whose variable is being gotten, or the turtle who is getting
    /// the variable of the current patch.
    pub agent: NodeId,
    /// The variable to get.
    pub var: PatchVarDesc,
}

impl EffectfulNode for GetPatchVarAsTurtleOrPatch {
    fn has_side_effects(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.agent]
    }

    fn output_type(
        &self,
        workspace: &Workspace,
        _nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<NetlogoInternalType> {
        Some(patch::patch_var_type(&workspace.world.patches.schema(), self.var))
    }
}

#[derive(Debug, Display)]
#[display("SetPatchVarAsTurtleOrPatch {var:?}")]
pub struct SetPatchVarAsTurtleOrPatch {
    /// The execution context to use.
    pub context: NodeId,
    /// The patch whose variable is being set, or the turtle who is setting
    /// the variable of the current patch.
    pub agent: NodeId,
    /// The variable to set.
    pub var: PatchVarDesc,
    /// The value to set the variable to.
    pub value: NodeId,
}

impl EffectfulNode for SetPatchVarAsTurtleOrPatch {
    fn has_side_effects(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.agent, self.value]
    }

    fn output_type(
        &self,
        _workspace: &Workspace,
        _nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<NetlogoInternalType> {
        None
    }
}
