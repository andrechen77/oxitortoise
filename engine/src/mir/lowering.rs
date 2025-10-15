use std::mem;

use derive_more::derive::Display;

use crate::{
    mir::{
        EffectfulNode, Function, NetlogoAbstractAbstractType, NodeId, Nodes, Program, StatementKind,
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

    fn output_type(
        &self,
        _program: &Program,
        _function: &Function,
        _nodes: &Nodes,
    ) -> NetlogoAbstractAbstractType {
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
    let statement_block = &function.cfg;
    for statement in &statement_block.statements {
        match statement {
            StatementKind::Node(node_id) => {
                lower_node_recursive(*node_id, program, function, &function.nodes)
            }
            _ => todo!(),
        }
    }
}

fn lower_node_recursive(
    node_id: NodeId,
    program: &Program,
    function: &Function,
    nodes: &RefCell<Nodes>,
) {
    let node = {
        let mut nodes_borrow = nodes.borrow_mut();
        mem::replace(&mut nodes_borrow[node_id], Box::new(Placeholder {}))
    };
    let expanded = node.lowering_expand(node_id, program, function, nodes);
    if !expanded {
        // lowering couldn't be done, to put the original node back
        nodes.borrow_mut()[node_id] = node;
    };

    // try to expand its children
    let dependencies = nodes.borrow()[node_id].dependencies();
    for dependency in dependencies {
        lower_node_recursive(dependency, program, function, nodes);
    }
}
