use std::{any::Any, mem};

use derive_more::derive::Display;
use tracing::debug;

use crate::{
    mir::{
        EffectfulNode, Function, FunctionId, MirType, MirVisitor, NetlogoAbstractType, NodeId,
        Nodes, Program, node, visit_mir_function,
    },
    sim::{turtle::TurtleVarDesc, value::NetlogoMachineType},
    util::cell::RefCell,
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
        program: &Program,
        fn_id: FunctionId,
        my_node_id: NodeId,
    ) -> Option<Box<dyn Fn(&Program, FunctionId, NodeId) -> bool>> {
        let _ = program;
        let _ = fn_id;
        let _ = my_node_id;
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

            debug!("Transforming node {:?}", node);

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

            debug!("Transforming node {:?}", node);

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
