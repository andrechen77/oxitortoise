//! Primitivees for moving turtles.

use derive_more::derive::Display;

use crate::{
    exec::jit::host_fn,
    mir::{
        Function, FunctionId, MirTy, NlAbstractTy, Node, NodeId, NodeKind, NodeTransform, Nodes,
        Program, WriteLirError, build_lir::LirInsnBuilder, node,
    },
};
use lir::smallvec::smallvec;

#[derive(Debug, Display)]
#[display("TurtleRotate")]
pub struct TurtleRotate {
    /// The execution context to use.
    pub context: NodeId,
    /// The turtle to rotate.
    pub turtle: NodeId,
    /// The amount to rotate.
    pub angle: NodeId,
}

impl Node for TurtleRotate {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.turtle, self.angle]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Unit)
    }

    fn write_lir_execution(
        &self,
        program: &Program,
        function: &Function,
        nodes: &Nodes,
        _my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[ctx_ptr] = lir_builder.get_node_results(program, function, nodes, self.context)
        else {
            panic!("expected node outputting context pointer to be a single LIR value")
        };
        let &[turtle_id] = lir_builder.get_node_results(program, function, nodes, self.turtle)
        else {
            panic!("expected node outputting turtle id to be a single LIR value")
        };
        let &[angle] = lir_builder.get_node_results(program, function, nodes, self.angle) else {
            panic!("expected node outputting angle to be a single LIR value")
        };
        lir_builder.push_lir_insn(lir::generate_host_function_call(
            host_fn::ROTATE_TURTLE,
            Box::new([ctx_ptr, turtle_id, angle]),
        ));
        Ok(())
    }
}

#[derive(Debug, Display)]
#[display("TurtleForward")]
pub struct TurtleForward {
    /// The execution context to use.
    pub context: NodeId,
    /// The turtle to move.
    pub turtle: NodeId,
    /// The distance to move.
    pub distance: NodeId,
}

impl Node for TurtleForward {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.turtle, self.distance]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Unit)
    }

    fn write_lir_execution(
        &self,
        program: &Program,
        function: &Function,
        nodes: &Nodes,
        _my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[ctx_ptr] = lir_builder.get_node_results(program, function, nodes, self.context)
        else {
            panic!("expected node outputting context pointer to be a single LIR value")
        };
        let &[turtle_id] = lir_builder.get_node_results(program, function, nodes, self.turtle)
        else {
            panic!("expected node outputting turtle id to be a single LIR value")
        };
        let &[distance] = lir_builder.get_node_results(program, function, nodes, self.distance)
        else {
            panic!("expected node outputting distance to be a single LIR value")
        };
        lir_builder.push_lir_insn(lir::generate_host_function_call(
            host_fn::TURTLE_FORWARD,
            Box::new([ctx_ptr, turtle_id, distance]),
        ));
        Ok(())
    }
}

#[derive(Debug, Display)]
#[display("CanMove")]
pub struct CanMove {
    /// The execution context to use.
    pub context: NodeId,
    /// The turtle to check movement for
    pub turtle: NodeId,
    /// The distance to check
    pub distance: NodeId,
}

impl Node for CanMove {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.turtle, self.distance]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Boolean)
    }

    fn peephole_transform(
        &self,
        _program: &Program,
        _fn_id: FunctionId,
        _my_node_id: NodeId,
    ) -> Option<NodeTransform> {
        fn break_down_can_move(program: &Program, fn_id: FunctionId, my_node_id: NodeId) -> bool {
            let function = program.functions[fn_id].borrow();
            let mut nodes = function.nodes.borrow_mut();
            let &CanMove { context, turtle, distance } = (&nodes[my_node_id]).try_into().unwrap();

            let patch = nodes.insert(NodeKind::from(node::PatchRelative {
                context,
                relative_loc: PatchLocRelation::Ahead,
                turtle,
                distance,
            }));

            nodes[my_node_id] =
                NodeKind::from(node::CheckNobody { context, agent: patch, negate: true });

            true
        }
        Some(Box::new(break_down_can_move))
    }
}

#[derive(Debug, Display)]
pub enum PatchLocRelation {
    #[display("Ahead")]
    Ahead,
    #[display("LeftAhead")]
    LeftAhead(NodeId),
    #[display("RightAhead")]
    RightAhead(NodeId),
}

#[derive(Debug, Display)]
#[display("PatchNearby {relative_loc:?}")]
pub struct PatchRelative {
    /// The execution context to use.
    pub context: NodeId,
    /// The location to check relative to the patch
    pub relative_loc: PatchLocRelation,
    /// The turtle to check from
    pub turtle: NodeId,
    /// The distance to check
    pub distance: NodeId,
}

impl Node for PatchRelative {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        let mut deps = vec![self.context, self.turtle, self.distance];
        match &self.relative_loc {
            PatchLocRelation::LeftAhead(heading) => deps.push(*heading),
            PatchLocRelation::RightAhead(heading) => deps.push(*heading),
            _ => (),
        }
        deps
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Patch)
    }

    // TODO(mvp) add host function for right_and_ahead, and add transformation
    // to turn left_and_ahead into right_and_ahead with negated operand

    fn write_lir_execution(
        &self,
        program: &Program,
        function: &Function,
        nodes: &Nodes,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[ctx_ptr] = lir_builder.get_node_results(program, function, nodes, self.context)
        else {
            panic!("expected node outputting context pointer to be a single LIR value")
        };
        let &[turtle_id] = lir_builder.get_node_results(program, function, nodes, self.turtle)
        else {
            panic!("expected node outputting turtle id to be a single LIR value")
        };
        let &[distance] = lir_builder.get_node_results(program, function, nodes, self.distance)
        else {
            panic!("expected node outputting distance to be a single LIR value")
        };

        let pc = match &self.relative_loc {
            PatchLocRelation::Ahead => lir_builder.push_lir_insn(lir::generate_host_function_call(
                host_fn::PATCH_AHEAD,
                Box::new([ctx_ptr, turtle_id, distance]),
            )),
            PatchLocRelation::RightAhead(angle) => {
                let &[angle] = lir_builder.get_node_results(program, function, nodes, *angle)
                else {
                    panic!("expected node outputting angle to be a single LIR value");
                };
                lir_builder.push_lir_insn(lir::generate_host_function_call(
                    host_fn::PATCH_RIGHT_AND_AHEAD,
                    Box::new([ctx_ptr, turtle_id, angle, distance]),
                ))
            }
            PatchLocRelation::LeftAhead(_) => {
                unimplemented!("transform the node to right-and-ahead with negated angle")
            }
        };

        lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(pc, 0)]);
        Ok(())
    }
}
