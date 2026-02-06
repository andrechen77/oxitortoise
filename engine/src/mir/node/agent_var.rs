//! Nodes for getting and setting agent and global variables.

use std::mem::offset_of;

use derive_more::derive::Display;
use lir::smallvec::smallvec;

use crate::{
    exec::{CanonExecutionContext, jit::InstallLir},
    mir::{
        FunctionId, MirTy, NlAbstractTy, Node, NodeId, NodeKind, NodeTransform, Program,
        WriteLirError, build_lir::LirInsnBuilder, node,
    },
    sim::{
        observer::calc_global_var_offset,
        patch::{PatchVarDesc, calc_patch_var_offset},
        turtle::{TurtleVarDesc, calc_turtle_var_offset},
        value::AGENT_INDEX_CONCRETE_TY,
        world::World,
    },
    util::reflection::Reflect,
    workspace::Workspace,
};

#[derive(Debug, Display, Copy, Clone)]
#[display("GetGlobalVar {index:?}")]
pub struct GetGlobalVar {
    pub context: NodeId,
    pub index: usize,
}

impl Node for GetGlobalVar {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context]
    }

    fn output_type(&self, program: &Program, _fn_id: FunctionId) -> MirTy {
        let Some(var) = program.globals.get(self.index) else {
            panic!("Unknown global var index: {:?}", self.index);
        };
        var.ty.clone()
    }

    fn lowering_expand(
        &self,
        _program: &Program,
        _fn_id: FunctionId,
        _my_node_id: NodeId,
    ) -> Option<NodeTransform> {
        fn lower_get_global_var(
            program: &mut Program,
            fn_id: FunctionId,
            my_node_id: NodeId,
        ) -> bool {
            let NodeKind::GetGlobalVar(my_node) = program.nodes[my_node_id] else {
                return false;
            };

            let (data_row, field_offset) =
                context_to_global_data(program, my_node.context, my_node.index);

            let field = NodeKind::from(node::MemLoad {
                ptr: data_row,
                offset: field_offset,
                ty: my_node.output_type(program, fn_id).repr(),
            });
            program.nodes[my_node_id] = field;
            true
        }

        Some(Box::new(lower_get_global_var))
    }
}

fn context_to_global_data(program: &mut Program, context: NodeId, var: usize) -> (NodeId, usize) {
    let (buffer_offset, field_offset) = calc_global_var_offset(program, var);

    // insert a node that gets the workspace pointer
    let workspace_ptr = program.nodes.insert(NodeKind::from(node::MemLoad {
        ptr: context,
        offset: offset_of!(CanonExecutionContext, workspace),
        ty: <*mut u8 as Reflect>::CONCRETE_TY,
    }));

    // insert a node that gets the row buffer
    let globals = program.nodes.insert(NodeKind::from(node::MemLoad {
        ptr: workspace_ptr,
        offset: offset_of!(Workspace, world) + offset_of!(World, globals) + buffer_offset,
        ty: <*mut u8 as Reflect>::CONCRETE_TY,
    }));

    (globals, field_offset)
}

// TODO(mvp) if this is a variable that may or may not exist depending on the
// breed, then we should check the breed of the turtle as well

#[derive(Debug, Display, Copy, Clone)]
#[display("GetTurtleVar {var:?}")]
pub struct GetTurtleVar {
    /// The execution context to use.
    pub context: NodeId,
    /// The turtle whose variable is being gotten.
    pub turtle: NodeId,
    /// The variable to get.
    pub var: TurtleVarDesc,
}

impl Node for GetTurtleVar {
    // Not pure!  Its value depends on `set` calls within the same block.  --Jason B. (11/12/25)
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.turtle]
    }

    fn output_type(&self, program: &Program, _fn_id: FunctionId) -> MirTy {
        // TODO(wishlist) this should probably be refactored into a function
        match self.var {
            TurtleVarDesc::Who => NlAbstractTy::Float.into(),
            TurtleVarDesc::Color => NlAbstractTy::Color.into(),
            TurtleVarDesc::Size => NlAbstractTy::Float.into(),
            TurtleVarDesc::Pos => NlAbstractTy::Point.into(),
            TurtleVarDesc::Xcor => NlAbstractTy::Float.into(),
            TurtleVarDesc::Ycor => NlAbstractTy::Float.into(),
            TurtleVarDesc::Custom(field) => program.custom_turtle_vars[field].ty.clone(),
        }
    }

    fn lowering_expand(
        &self,
        _program: &Program,
        _fn_id: FunctionId,
        _my_node_id: NodeId,
    ) -> Option<NodeTransform> {
        fn lower_get_turtle_var(
            program: &mut Program,
            fn_id: FunctionId,
            my_node_id: NodeId,
        ) -> bool {
            let NodeKind::GetTurtleVar(my_node) = program.nodes[my_node_id] else {
                return false;
            };

            let (data_row, field_offset) =
                context_to_turtle_data(program, my_node.context, my_node.turtle, my_node.var);

            // create a node to get the field
            let field = NodeKind::from(node::MemLoad {
                ptr: data_row,
                offset: field_offset,
                ty: my_node.output_type(program, fn_id).repr(),
            });
            program.nodes[my_node_id] = field;
            true
        }

        Some(Box::new(lower_get_turtle_var))
    }
}

#[derive(Debug, Display, Copy, Clone)]
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

impl Node for SetTurtleVar {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.turtle, self.value]
    }
    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        NlAbstractTy::Unit.into()
    }

    fn lowering_expand(
        &self,
        _program: &Program,
        _fn_id: FunctionId,
        _my_node_id: NodeId,
    ) -> Option<NodeTransform> {
        fn lower_set_turtle_var(
            program: &mut Program,
            _fn_id: FunctionId,
            my_node_id: NodeId,
        ) -> bool {
            let NodeKind::SetTurtleVar(my_node) = program.nodes[my_node_id] else {
                return false;
            };

            let (data_row, field_offset) =
                context_to_turtle_data(program, my_node.context, my_node.turtle, my_node.var);

            // create a node to set the field
            let field = NodeKind::from(node::MemStore {
                ptr: data_row,
                offset: field_offset,
                value: my_node.value,
            });
            program.nodes[my_node_id] = field;
            true
        }

        Some(Box::new(lower_set_turtle_var))
    }
}

/// Helper function to derive a pointer to the data row of a turtle's variable.
/// Returns the NodeId of the node that outputs the pointer to data row, as well
/// as the byte offset of the field within the data row. This is used by both
/// loads and stores.
fn context_to_turtle_data(
    program: &mut Program,
    context: NodeId,
    turtle_id: NodeId,
    var: TurtleVarDesc,
) -> (NodeId, usize) {
    let (buffer_offset, stride, field_offset): (usize, usize, usize) =
        calc_turtle_var_offset(program, var);

    // insert a node that gets the workspace pointer
    let workspace_ptr = program.nodes.insert(NodeKind::from(node::MemLoad {
        ptr: context,
        offset: offset_of!(CanonExecutionContext, workspace),
        ty: <*mut u8 as Reflect>::CONCRETE_TY,
    }));

    // insert a node that gets the row buffer
    let row_buffer = program.nodes.insert(NodeKind::from(node::MemLoad {
        ptr: workspace_ptr,
        offset: offset_of!(Workspace, world) + offset_of!(World, turtles) + buffer_offset,
        ty: <*mut u8 as Reflect>::CONCRETE_TY,
    }));

    // insert a node that gets the agent index
    let turtle_idx = program.nodes.insert(NodeKind::from(node::TurtleIdToIndex { turtle_id }));

    // insert a node that gets the right data row
    let data_row = program.nodes.insert(NodeKind::from(node::DeriveElement {
        ptr: row_buffer,
        index: turtle_idx,
        stride,
    }));

    (data_row, field_offset)
}

#[derive(Debug, Display)]
#[display("TurtleIdToIndex")]
pub struct TurtleIdToIndex {
    /// The turtle id to convert.
    pub turtle_id: NodeId,
}

impl Node for TurtleIdToIndex {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.turtle_id]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        AGENT_INDEX_CONCRETE_TY.into()
    }

    fn write_lir_execution<I: InstallLir>(
        &self,
        program: &Program,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[turtle_id] = lir_builder.get_node_results::<I>(program, self.turtle_id) else {
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

#[derive(Debug, Display, Copy, Clone)]
#[display("GetPatchVar {var:?}")]
pub struct GetPatchVar {
    /// The execution context to use.
    pub context: NodeId,
    /// The patch whose variable is being gotten.
    pub patch: NodeId,
    /// The variable to get.
    pub var: PatchVarDesc,
}

impl Node for GetPatchVar {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.patch]
    }

    fn output_type(&self, program: &Program, _fn_id: FunctionId) -> MirTy {
        match self.var {
            PatchVarDesc::Pcolor => NlAbstractTy::Color.into(),
            PatchVarDesc::Pos => NlAbstractTy::Point.into(),
            PatchVarDesc::Custom(field) => program.custom_patch_vars[field].ty.clone(),
        }
    }

    fn lowering_expand(
        &self,
        _program: &Program,
        _fn_id: FunctionId,
        _my_node_id: NodeId,
    ) -> Option<NodeTransform> {
        fn lower_get_patch_var(
            program: &mut Program,
            fn_id: FunctionId,
            my_node_id: NodeId,
        ) -> bool {
            let NodeKind::GetPatchVar(my_node) = program.nodes[my_node_id] else {
                return false;
            };
            let GetPatchVar { context, patch, var } = my_node;

            let (data_row, field_offset) = context_to_patch_data(program, context, patch, var);

            // create a node to get the field
            let field = NodeKind::from(node::MemLoad {
                ptr: data_row,
                offset: field_offset,
                ty: my_node.output_type(program, fn_id).repr(),
            });
            program.nodes[my_node_id] = field;

            true
        }
        Some(Box::new(lower_get_patch_var))
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

impl Node for SetPatchVar {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.patch, self.value]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        NlAbstractTy::Unit.into()
    }

    fn lowering_expand(
        &self,
        _program: &Program,
        _fn_id: FunctionId,
        _my_node_id: NodeId,
    ) -> Option<NodeTransform> {
        fn lower_set_patch_var(
            program: &mut Program,
            _fn_id: FunctionId,
            my_node_id: NodeId,
        ) -> bool {
            let NodeKind::SetPatchVar(SetPatchVar { context, patch, var, value }) =
                program.nodes[my_node_id]
            else {
                return false;
            };

            let (data_row, field_offset) = context_to_patch_data(program, context, patch, var);

            // create a node to set the field
            let field =
                NodeKind::from(node::MemStore { ptr: data_row, offset: field_offset, value });
            program.nodes[my_node_id] = field;
            true
        }

        Some(Box::new(lower_set_patch_var))
    }
}

/// Helper function to derive a pointer to the data row of a patch's variable. Returns the
/// the NodeId of the node that outputs the pointer to data row, as well as the byte
/// offset of the field within the data row. This is used by both loads and stores.
fn context_to_patch_data(
    program: &mut Program,
    context: NodeId,
    patch_id: NodeId,
    var: PatchVarDesc,
) -> (NodeId, usize) {
    let (buffer_offset, stride, field_offset) = calc_patch_var_offset(program, var);

    // insert a node that gets the workspace pointer
    let workspace_ptr = program.nodes.insert(NodeKind::from(node::MemLoad {
        ptr: context,
        offset: offset_of!(CanonExecutionContext, workspace),
        ty: <*mut u8 as Reflect>::CONCRETE_TY,
    }));

    // insert a node that gets the row buffer
    let row_buffer = program.nodes.insert(NodeKind::from(node::MemLoad {
        ptr: workspace_ptr,
        offset: offset_of!(Workspace, world) + offset_of!(World, patches) + buffer_offset,
        ty: <*mut u8 as Reflect>::CONCRETE_TY,
    }));

    // insert a node that gets the agent index
    // let patch_idx = program.nodes.insert(Box::new(node::PatchIdToIndex { patch_id }));

    // insert a node that gets the right data row
    let data_row = program.nodes.insert(NodeKind::from(node::DeriveElement {
        ptr: row_buffer,
        index: patch_id,
        stride,
    }));

    (data_row, field_offset)
}

/// A node for getting an patch variable when the type of the agent is unknown.
#[derive(Debug, Display, Clone)]
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

impl Node for GetPatchVarAsTurtleOrPatch {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.agent]
    }

    fn output_type(&self, program: &Program, _fn_id: FunctionId) -> MirTy {
        // TODO(wishlist) refactor to deduplicate with GetPatchVar
        match self.var {
            PatchVarDesc::Pcolor => NlAbstractTy::Color.into(),
            PatchVarDesc::Pos => NlAbstractTy::Point.into(),
            PatchVarDesc::Custom(field) => program.custom_patch_vars[field].ty.clone(),
        }
    }

    fn peephole_transform(
        &self,
        _program: &Program,
        _fn_id: FunctionId,
        _my_node_id: NodeId,
    ) -> Option<NodeTransform> {
        fn decompose_get_patch_var(
            program: &mut Program,
            fn_id: FunctionId,
            my_node_id: NodeId,
        ) -> bool {
            let NodeKind::GetPatchVarAsTurtleOrPatch(GetPatchVarAsTurtleOrPatch {
                context,
                agent,
                var,
            }) = program.nodes[my_node_id]
            else {
                return false;
            };

            match program.nodes[agent]
                .output_type(program, fn_id)
                .abstr
                .expect("agent must have an abstract type")
            {
                NlAbstractTy::Patch => {
                    program.nodes[my_node_id] =
                        NodeKind::from(node::GetPatchVar { context, patch: agent, var });
                    true
                }
                NlAbstractTy::Turtle => {
                    let xcor = program.nodes.insert(NodeKind::from(node::GetTurtleVar {
                        context,
                        turtle: agent,
                        var: TurtleVarDesc::Xcor,
                    }));

                    let ycor = program.nodes.insert(NodeKind::from(node::GetTurtleVar {
                        context,
                        turtle: agent,
                        var: TurtleVarDesc::Ycor,
                    }));

                    let patch_here = program.nodes.insert(NodeKind::from(node::PatchAt {
                        context,
                        x: xcor,
                        y: ycor,
                    }));

                    program.nodes[my_node_id] =
                        NodeKind::from(node::GetPatchVar { context, patch: patch_here, var });

                    true
                }
                _ => false,
            }
        }

        Some(Box::new(decompose_get_patch_var))
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

impl Node for SetPatchVarAsTurtleOrPatch {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.agent, self.value]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        NlAbstractTy::Unit.into()
    }

    fn peephole_transform(
        &self,
        _program: &Program,
        _fn_id: FunctionId,
        _my_node_id: NodeId,
    ) -> Option<NodeTransform> {
        let agent = self.agent;
        let context = self.context;
        let var = self.var;
        let value = self.value;
        let transform =
            move |program: &mut Program, fn_id: FunctionId, my_node_id: NodeId| match program.nodes
                [agent]
                .output_type(program, fn_id)
                .abstr
                .unwrap()
            {
                NlAbstractTy::Patch => {
                    program.nodes[my_node_id] =
                        NodeKind::from(node::SetPatchVar { context, patch: agent, var, value });
                    true
                }
                NlAbstractTy::Turtle => {
                    let xcor = program.nodes.insert(NodeKind::from(node::GetTurtleVar {
                        context,
                        turtle: agent,
                        var: TurtleVarDesc::Xcor,
                    }));

                    let ycor = program.nodes.insert(NodeKind::from(node::GetTurtleVar {
                        context,
                        turtle: agent,
                        var: TurtleVarDesc::Ycor,
                    }));

                    let patch_here = program.nodes.insert(NodeKind::from(node::PatchAt {
                        context,
                        x: xcor,
                        y: ycor,
                    }));

                    program.nodes[my_node_id] = NodeKind::from(node::SetPatchVar {
                        context,
                        patch: patch_here,
                        var,
                        value,
                    });

                    true
                }
                _ => false,
            };
        Some(Box::new(transform))
    }
}
