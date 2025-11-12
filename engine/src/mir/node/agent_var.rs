use std::mem::offset_of;

use derive_more::derive::Display;
use lir::smallvec::smallvec;
use slotmap::SlotMap;

use crate::{
    exec::CanonExecutionContext,
    mir::{
        EffectfulNode, EffectfulNodeKind, Function, FunctionId, MirTy, NlAbstractTy, NodeId,
        NodeTransform, Nodes, Program, WriteLirError, build_lir::LirInsnBuilder, node,
    },
    sim::{
        patch::{PatchVarDesc, calc_patch_var_offset},
        turtle::{TurtleVarDesc, calc_turtle_var_offset},
        value::NlMachineTy,
    },
    workspace::Workspace,
};

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

impl EffectfulNode for GetTurtleVar {
    // Not pure!  Its value depends on `set` calls within the same block.  --Jason B. (11/12/25)
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.turtle]
    }

    fn output_type(&self, program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        // TODO(wishlist) this should probably be refactored into a function
        MirTy::Abstract(match self.var {
            TurtleVarDesc::Who => NlAbstractTy::Float,
            TurtleVarDesc::Color => NlAbstractTy::Color,
            TurtleVarDesc::Size => NlAbstractTy::Float,
            TurtleVarDesc::Xcor => NlAbstractTy::Float,
            TurtleVarDesc::Ycor => NlAbstractTy::Float,
            TurtleVarDesc::Custom(field) => program.custom_turtle_vars[field].ty.clone(),
        })
    }

    fn lowering_expand(
        &self,
        _program: &Program,
        _fn_id: FunctionId,
        _my_node_id: NodeId,
    ) -> Option<NodeTransform> {
        fn lower_get_turtle_var(program: &Program, fn_id: FunctionId, my_node_id: NodeId) -> bool {
            let function = program.functions[fn_id].borrow();
            let EffectfulNodeKind::GetTurtleVar(my_node) = function.nodes.borrow()[my_node_id]
            else {
                panic!("expected node to be a GetTurtleVar");
            };

            let (data_row, field_offset) = context_to_turtle_data(
                &mut function.nodes.borrow_mut(),
                program,
                my_node.context,
                my_node.turtle,
                my_node.var,
            );

            // create a node to get the field
            let field = EffectfulNodeKind::from(node::MemLoad {
                ptr: data_row,
                offset: field_offset,
                ty: my_node.output_type(program, &function, &function.nodes.borrow()).repr(),
            });
            function.nodes.borrow_mut()[my_node_id] = field;
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

impl EffectfulNode for SetTurtleVar {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.turtle, self.value]
    }
    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Unit)
    }

    fn lowering_expand(
        &self,
        _program: &Program,
        _fn_id: FunctionId,
        _my_node_id: NodeId,
    ) -> Option<NodeTransform> {
        fn lower_set_turtle_var(program: &Program, fn_id: FunctionId, my_node_id: NodeId) -> bool {
            let function = program.functions[fn_id].borrow();
            let EffectfulNodeKind::SetTurtleVar(my_node) = function.nodes.borrow()[my_node_id]
            else {
                panic!("expected node to be a SetTurtleVar");
            };

            let (data_row, field_offset) = context_to_turtle_data(
                &mut function.nodes.borrow_mut(),
                program,
                my_node.context,
                my_node.turtle,
                my_node.var,
            );

            // create a node to set the field
            let field = EffectfulNodeKind::from(node::MemStore {
                ptr: data_row,
                offset: field_offset,
                value: my_node.value,
            });
            function.nodes.borrow_mut()[my_node_id] = field;
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
    nodes: &mut SlotMap<NodeId, EffectfulNodeKind>,
    program: &Program,
    context: NodeId,
    turtle_id: NodeId,
    var: TurtleVarDesc,
) -> (NodeId, usize) {
    let (buffer_offset, stride, field_offset): (usize, usize, usize) =
        calc_turtle_var_offset(program, var);

    // insert a node that gets the workspace pointer
    let workspace_ptr = nodes.insert(EffectfulNodeKind::from(node::MemLoad {
        ptr: context,
        offset: offset_of!(CanonExecutionContext, workspace),
        ty: NlMachineTy::UNTYPED_PTR,
    }));

    // insert a node that gets the row buffer
    let row_buffer = nodes.insert(EffectfulNodeKind::from(node::MemLoad {
        ptr: workspace_ptr,
        offset: offset_of!(Workspace, world) + buffer_offset,
        ty: NlMachineTy::UNTYPED_PTR,
    }));

    // insert a node that gets the agent index
    let turtle_idx = nodes.insert(EffectfulNodeKind::from(node::TurtleIdToIndex { turtle_id }));

    // insert a node that gets the right data row
    let data_row = nodes.insert(EffectfulNodeKind::from(node::DeriveElement {
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

impl EffectfulNode for TurtleIdToIndex {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.turtle_id]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Machine(NlMachineTy::AGENT_INDEX)
    }

    fn write_lir_execution(
        &self,
        my_node_id: NodeId,
        nodes: &SlotMap<NodeId, EffectfulNodeKind>,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
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

impl EffectfulNode for GetPatchVar {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.patch]
    }

    fn output_type(&self, program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        match self.var {
            PatchVarDesc::Pcolor => MirTy::Abstract(NlAbstractTy::Color),
            PatchVarDesc::Custom(field) => {
                MirTy::Abstract(program.custom_patch_vars[field].ty.clone())
            }
        }
    }

    fn lowering_expand(
        &self,
        _program: &Program,
        _fn_id: FunctionId,
        _my_node_id: NodeId,
    ) -> Option<NodeTransform> {
        fn lower_get_patch_var(program: &Program, fn_id: FunctionId, my_node_id: NodeId) -> bool {
            let function = program.functions[fn_id].borrow();
            let EffectfulNodeKind::GetPatchVar(my_node) = function.nodes.borrow()[my_node_id]
            else {
                panic!("expected node to be a GetPatchVar");
            };
            let GetPatchVar { context, patch, var } = my_node;

            let (data_row, field_offset) = context_to_patch_data(
                &mut function.nodes.borrow_mut(),
                program,
                context,
                patch,
                var,
            );

            // create a node to get the field
            let field = EffectfulNodeKind::from(node::MemLoad {
                ptr: data_row,
                offset: field_offset,
                ty: my_node.output_type(program, &function, &function.nodes.borrow()).repr(),
            });
            function.nodes.borrow_mut()[my_node_id] = field;

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

impl EffectfulNode for SetPatchVar {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.patch, self.value]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Unit)
    }

    fn lowering_expand(
        &self,
        _program: &Program,
        _fn_id: FunctionId,
        _my_node_id: NodeId,
    ) -> Option<NodeTransform> {
        fn lower_set_patch_var(program: &Program, fn_id: FunctionId, my_node_id: NodeId) -> bool {
            let function = program.functions[fn_id].borrow();
            let EffectfulNodeKind::SetPatchVar(SetPatchVar { context, patch, var, value }) =
                function.nodes.borrow()[my_node_id]
            else {
                panic!("expected node to be a SetPatchVar");
            };

            let (data_row, field_offset) = context_to_patch_data(
                &mut function.nodes.borrow_mut(),
                program,
                context,
                patch,
                var,
            );

            // create a node to set the field
            let field = EffectfulNodeKind::from(node::MemStore {
                ptr: data_row,
                offset: field_offset,
                value,
            });
            function.nodes.borrow_mut()[my_node_id] = field;
            true
        }

        Some(Box::new(lower_set_patch_var))
    }
}

/// Helper function to derive a pointer to the data row of a patch's variable. Returns the
/// the NodeId of the node that outputs the pointer to data row, as well as the byte
/// offset of the field within the data row. This is used by both loads and stores.
fn context_to_patch_data(
    nodes: &mut SlotMap<NodeId, EffectfulNodeKind>,
    mir: &Program,
    context: NodeId,
    patch_id: NodeId,
    var: PatchVarDesc,
) -> (NodeId, usize) {
    let (buffer_offset, stride, field_offset) = calc_patch_var_offset(mir, var);

    // insert a node that gets the workspace pointer
    let workspace_ptr = nodes.insert(EffectfulNodeKind::from(node::MemLoad {
        ptr: context,
        offset: offset_of!(CanonExecutionContext, workspace),
        ty: NlMachineTy::UNTYPED_PTR,
    }));

    // insert a node that gets the row buffer
    let row_buffer = nodes.insert(EffectfulNodeKind::from(node::MemLoad {
        ptr: workspace_ptr,
        offset: offset_of!(Workspace, world) + buffer_offset,
        ty: NlMachineTy::UNTYPED_PTR,
    }));

    // insert a node that gets the agent index
    // let patch_idx = nodes.insert(Box::new(node::PatchIdToIndex { patch_id }));

    // insert a node that gets the right data row
    let data_row = nodes.insert(EffectfulNodeKind::from(node::DeriveElement {
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

impl EffectfulNode for GetPatchVarAsTurtleOrPatch {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.agent]
    }

    fn output_type(&self, program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        // TODO(wishlist) refactor to deduplicate with GetPatchVar
        match self.var {
            PatchVarDesc::Custom(field) => {
                MirTy::Abstract(program.custom_patch_vars[field].ty.clone())
            }
            PatchVarDesc::Pcolor => MirTy::Abstract(NlAbstractTy::Color),
        }
    }

    fn peephole_transform(
        &self,
        _program: &Program,
        _fn_id: FunctionId,
        _my_node_id: NodeId,
    ) -> Option<NodeTransform> {
        Some(Box::new(decompose_get_patch_var))
    }
}

fn decompose_get_patch_var(program: &Program, fn_id: FunctionId, my_node_id: NodeId) -> bool {
    let function = program.functions[fn_id].borrow();
    let nodes_borrowed = function.nodes.borrow();

    let EffectfulNodeKind::GetPatchVarAsTurtleOrPatch(GetPatchVarAsTurtleOrPatch {
        context,
        agent,
        var,
    }) = nodes_borrowed[my_node_id]
    else {
        panic!("expected node to be a GetPatchVarAsTurtleOrPatch");
    };

    match nodes_borrowed[agent].output_type(program, &function, &nodes_borrowed) {
        MirTy::Abstract(NlAbstractTy::Patch) => {
            drop(nodes_borrowed);
            function.nodes.borrow_mut()[my_node_id] =
                EffectfulNodeKind::from(node::GetPatchVar { context, patch: agent, var });
            true
        }
        MirTy::Abstract(NlAbstractTy::Turtle) => {
            drop(nodes_borrowed);
            let mut nodes = function.nodes.borrow_mut();

            let xcor = nodes.insert(EffectfulNodeKind::from(node::GetTurtleVar {
                context,
                turtle: agent,
                var: TurtleVarDesc::Xcor,
            }));

            let ycor = nodes.insert(EffectfulNodeKind::from(node::GetTurtleVar {
                context,
                turtle: agent,
                var: TurtleVarDesc::Ycor,
            }));

            let patch_here =
                nodes.insert(EffectfulNodeKind::from(node::PatchAt { x: xcor, y: ycor }));

            nodes[my_node_id] =
                EffectfulNodeKind::from(node::GetPatchVar { context, patch: patch_here, var });

            true
        }
        _ => false,
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
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.agent, self.value]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Unit)
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
        let transform = move |program: &Program, fn_id: FunctionId, my_node_id: NodeId| {
            let function = program.functions[fn_id].borrow();
            let nodes_borrowed = function.nodes.borrow();
            match nodes_borrowed[agent].output_type(program, &function, &nodes_borrowed) {
                MirTy::Abstract(NlAbstractTy::Patch) => {
                    drop(nodes_borrowed);
                    function.nodes.borrow_mut()[my_node_id] =
                        EffectfulNodeKind::from(node::SetPatchVar {
                            context,
                            patch: agent,
                            var,
                            value,
                        });
                    true
                }
                MirTy::Abstract(NlAbstractTy::Turtle) => {
                    drop(nodes_borrowed);
                    let mut nodes = function.nodes.borrow_mut();

                    let xcor = nodes.insert(EffectfulNodeKind::from(node::GetTurtleVar {
                        context,
                        turtle: agent,
                        var: TurtleVarDesc::Xcor,
                    }));

                    let ycor = nodes.insert(EffectfulNodeKind::from(node::GetTurtleVar {
                        context,
                        turtle: agent,
                        var: TurtleVarDesc::Ycor,
                    }));

                    let patch_here =
                        nodes.insert(EffectfulNodeKind::from(node::PatchAt { x: xcor, y: ycor }));

                    nodes[my_node_id] = EffectfulNodeKind::from(node::SetPatchVar {
                        context,
                        patch: patch_here,
                        var,
                        value,
                    });

                    true
                }
                _ => false,
            }
        };
        Some(Box::new(transform))
    }
}
