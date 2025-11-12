use derive_more::derive::Display;
use tracing::trace;

use crate::mir::{
    ClosureType, EffectfulNode, EffectfulNodeKind, Function, FunctionId, MirType, MirVisitor,
    NodeId, NodeTransform, Nodes, Program, visit_mir_function,
};

#[derive(Debug, Display)]
#[display("Placeholder")]
pub struct Placeholder {}

impl EffectfulNode for Placeholder {
    fn has_side_effects(&self) -> bool {
        panic!()
    }

    fn dependencies(&self) -> Vec<NodeId> {
        panic!()
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirType {
        panic!()
    }

    fn lowering_expand(
        &self,
        _program: &Program,
        _fn_id: FunctionId,
        _my_node_id: NodeId,
    ) -> Option<NodeTransform> {
        None
    }
}

pub fn lower(program: &Program, fn_id: FunctionId) {
    struct Visitor<'a> {
        program: &'a Program,
        fn_id: FunctionId,
    }
    impl<'a> MirVisitor for Visitor<'a> {
        fn visit_node(&mut self, node_id: NodeId) {
            let function = self.program.functions[self.fn_id].borrow();
            let nodes = function.nodes.borrow();
            let node = &nodes[node_id];

            trace!("Transforming node {:?}", node);

            let transform = node.lowering_expand(self.program, self.fn_id, node_id);
            if let Some(transform) = transform {
                drop(nodes);
                drop(function);
                transform(self.program, self.fn_id, node_id);
            }
        }
    }
    visit_mir_function(&mut Visitor { program, fn_id }, &program.functions[fn_id].borrow());
}

pub fn peephole_transform(program: &Program, fn_id: FunctionId) {
    struct Visitor<'a> {
        program: &'a Program,
        fn_id: FunctionId,
    }
    impl<'a> MirVisitor for Visitor<'a> {
        fn visit_node(&mut self, node_id: NodeId) {
            let function = self.program.functions[self.fn_id].borrow();
            let nodes = function.nodes.borrow();
            let node = &nodes[node_id];

            trace!("Transforming node {:?}", node);

            let transform = node.peephole_transform(self.program, self.fn_id, node_id);
            if let Some(transform) = transform {
                drop(nodes);
                drop(function);
                transform(self.program, self.fn_id, node_id);
            }
        }
    }
    visit_mir_function(&mut Visitor { program, fn_id }, &program.functions[fn_id].borrow());
}

// HACK this optimization should instead be part of the type inference pass,
// which should collect information about all the types in the program; the
// types of the arguments to the `of`'s closure should be included in there.
// However, we are able to perform this optimization prematurely because we know
// that the `of` is the only way that the closure is ever going to be called, so
// we can assume that whatever is passed to `of` is going to be the only thing
// passed to the closure.
pub fn optimize_of_agent_type(program: &Program, fn_id: FunctionId) {
    struct Visitor<'a> {
        program: &'a Program,
        fn_id: FunctionId,
    }
    impl<'a> MirVisitor for Visitor<'a> {
        fn visit_node(&mut self, node_id: NodeId) {
            let function = self.program.functions[self.fn_id].borrow();
            let nodes = function.nodes.borrow();
            let node = &nodes[node_id];

            if let EffectfulNodeKind::Of(of) = &node {
                trace!("Optimizing Of node {:?}", node);

                let recipients = &nodes[of.recipients];
                let EffectfulNodeKind::Closure(closure) = &nodes[of.body] else {
                    return;
                };
                let mut body = self.program.functions[closure.body].borrow_mut();

                let self_param_id = body.parameters[ClosureType::PARAM_ARG_IDX];

                let ty = recipients.output_type(self.program, &function, &nodes).clone();
                body.locals[self_param_id].ty = ty;
            }
        }
    }
    visit_mir_function(&mut Visitor { program, fn_id }, &program.functions[fn_id].borrow());
}
