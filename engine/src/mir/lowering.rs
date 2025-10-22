use std::mem;

use derive_more::derive::Display;
use tracing::debug;

use crate::{
    mir::{
        EffectfulNode, Function, MirType, NodeId, Nodes, Program, StatementBlock, StatementKind,
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
    lower_statement_block_recursive(&function.cfg, program, function);
}

fn lower_statement_block_recursive(
    statement_block: &StatementBlock,
    program: &Program,
    function: &Function,
) {
    for statement in &statement_block.statements {
        match statement {
            StatementKind::Node(node_id) => {
                lower_node_recursive(*node_id, program, function, &function.nodes)
            }
            StatementKind::IfElse { condition, then_block, else_block } => {
                lower_node_recursive(*condition, program, function, &function.nodes);
                lower_statement_block_recursive(then_block, program, function);
                lower_statement_block_recursive(else_block, program, function);
            }
            StatementKind::Repeat { num_repetitions, block } => {
                lower_node_recursive(*num_repetitions, program, function, &function.nodes);
                lower_statement_block_recursive(block, program, function);
            }
            StatementKind::Return { value } => {
                lower_node_recursive(*value, program, function, &function.nodes);
            }
            StatementKind::Stop => {
                // do nothing
            }
        }
    }
}

fn lower_node_recursive(
    node_id: NodeId,
    program: &Program,
    function: &Function,
    nodes: &RefCell<Nodes>,
) {
    let node = mem::replace(&mut nodes.borrow_mut()[node_id], Box::new(Placeholder {}));

    debug!("Lowering node {:?}", node);

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
