use std::fmt;

use derive_more::derive::Display;
use tracing::trace;

use crate::mir::{
    ClosureType, FunctionId, LocalId, MirTy, MirVisitor, Node, NodeId, NodeKind, NodeTransform,
    Program, visit_mir_function,
};

#[derive(Debug, Display)]
#[display("Placeholder")]
pub struct Placeholder {}

impl Node for Placeholder {
    fn is_pure(&self) -> bool {
        panic!()
    }

    fn dependencies(&self) -> Vec<(&'static str, NodeId)> {
        panic!()
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
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

    fn pretty_print(&self, _program: &Program, mut out: impl fmt::Write) -> fmt::Result {
        write!(out, "Placeholder")
    }
}

pub fn lower(program: &mut Program, fn_id: FunctionId) {
    struct Visitor {
        transformations: Vec<(NodeTransform, NodeId)>,
        fn_id: FunctionId,
    }
    impl MirVisitor for Visitor {
        fn visit_node(&mut self, program: &Program, _fn_id: FunctionId, node_id: NodeId) {
            let node = &program.nodes[node_id];

            trace!("Visiting node {:?}", node);

            let transform = node.lowering_expand(program, self.fn_id, node_id);

            if let Some(transform) = transform {
                self.transformations.push((transform, node_id));
            }
        }
    }
    let mut visitor = Visitor { transformations: Vec::new(), fn_id };
    visit_mir_function(&mut visitor, program, fn_id);

    for (transform, node_id) in visitor.transformations {
        trace!("Applying lowering expansion to node {:?}", node_id);
        transform(program, fn_id, node_id);
    }
}

pub fn peephole_transform(program: &mut Program, fn_id: FunctionId) {
    struct Visitor {
        transformations: Vec<(NodeTransform, NodeId)>,
        fn_id: FunctionId,
    }
    impl MirVisitor for Visitor {
        fn visit_node(&mut self, program: &Program, _fn_id: FunctionId, node_id: NodeId) {
            let node = &program.nodes[node_id];

            trace!("Visiting node {:?}", node);

            let transform = node.peephole_transform(program, self.fn_id, node_id);
            if let Some(transform) = transform {
                self.transformations.push((transform, node_id));
            }
        }
    }
    let mut visitor = Visitor { transformations: Vec::new(), fn_id };
    visit_mir_function(&mut visitor, program, fn_id);

    for (transform, node_id) in visitor.transformations {
        trace!(
            "Applying peephole transformation to node {:?} {:?}",
            node_id, program.nodes[node_id]
        );
        transform(program, fn_id, node_id);
    }
}

// HACK this optimization should instead be part of the type inference pass,
// which should collect information about all the types in the program; the
// types of the arguments to the `of`'s closure should be included in there.
// However, we are able to perform this optimization prematurely because we know
// that the `of` is the only way that the closure is ever going to be called, so
// we can assume that whatever is passed to `of` is going to be the only thing
// passed to the closure.
pub fn optimize_of_agent_type(program: &mut Program, fn_id: FunctionId) {
    struct Visitor {
        type_changes: Vec<(LocalId, MirTy)>,
    }
    impl MirVisitor for Visitor {
        fn visit_node(&mut self, program: &Program, fn_id: FunctionId, node_id: NodeId) {
            let node = &program.nodes[node_id];

            if let NodeKind::Of(of) = &node {
                trace!("Optimizing Of node {:?}", node);

                let recipients =
                    &program.nodes[of.recipients.node().expect("TODO can't always unwrap here")];
                let NodeKind::Closure(closure) = &program.nodes[of.body] else {
                    return;
                };

                let self_param_id =
                    program.functions[closure.body].parameters[ClosureType::PARAM_ARG_IDX];

                let ty = recipients.output_type(program, fn_id).clone();
                self.type_changes.push((self_param_id, ty));
            }
        }
    }
    let mut visitor = Visitor { type_changes: Vec::new() };
    visit_mir_function(&mut visitor, program, fn_id);

    // apply the changes to the local variables
    for (local_id, ty) in visitor.type_changes {
        program.locals[local_id].ty = ty;
    }
}
