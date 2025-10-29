use std::mem;

use derive_more::derive::Display;
use tracing::debug;

use crate::{
    mir::{
        EffectfulNode, Function, MirType, MirVisitor, NodeId, Nodes, Program, visit_mir_function,
    },
    util::cell::RefCell,
};

#[derive(Debug, Display)]
#[display("Placeholder")]
struct Placeholder {}

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
        _my_node_id: NodeId,
        _program: &Program,
        _function: &Function,
        _nodes: &RefCell<Nodes>,
    ) -> bool {
        panic!()
    }
}

pub fn lower(function: &mut Function, program: &Program) {
    struct Visitor<'a> {
        program: &'a Program,
        function: &'a Function,
    }
    impl<'a> MirVisitor for Visitor<'a> {
        fn visit_node(&mut self, node_id: NodeId) {
            let node = mem::replace(
                &mut self.function.nodes.borrow_mut()[node_id],
                Box::new(Placeholder {}),
            );

            debug!("Lowering node {:?}", node);

            let expanded =
                node.lowering_expand(node_id, self.program, self.function, &self.function.nodes);
            if !expanded {
                // lowering couldn't be done, so put the original node back
                self.function.nodes.borrow_mut()[node_id] = node;
            }
        }
    }
    visit_mir_function(&mut Visitor { program, function }, function);
}

pub fn transform(function: &mut Function, program: &Program) {
    struct Visitor<'a> {
        program: &'a Program,
        function: &'a Function,
    }
    impl<'a> MirVisitor for Visitor<'a> {
        fn visit_node(&mut self, node_id: NodeId) {
            let node = mem::replace(
                &mut self.function.nodes.borrow_mut()[node_id],
                Box::new(Placeholder {}),
            );

            debug!("Transforming node {:?}", node);

            let expanded =
                node.transform(node_id, self.program, self.function, &self.function.nodes);
            if !expanded {
                // transformation couldn't be done, so put the original node back
                self.function.nodes.borrow_mut()[node_id] = node;
            }
        }
    }
    visit_mir_function(&mut Visitor { program, function }, function);
}
