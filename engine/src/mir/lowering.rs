use std::mem;

use derive_more::derive::Display;
use slotmap::SlotMap;

use crate::{
    mir::{EffectfulNode, Function, LocalDeclaration, LocalId, NodeId, StatementKind},
    sim::value::NetlogoInternalType,
    workspace::Workspace,
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
        _workspace: &Workspace,
        _nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<NetlogoInternalType> {
        panic!()
    }

    fn lowering_expand(
        &self,
        _my_node_id: NodeId,
        _workspace: &Workspace,
        _nodes: &mut SlotMap<NodeId, Box<dyn EffectfulNode>>,
    ) -> bool {
        panic!()
    }
}

pub fn lower(function: &mut Function, workspace: &Workspace) {
    let statement_block = &function.cfg;
    for statement in &statement_block.statements {
        match statement {
            StatementKind::Node(node_id) => {
                lower_node_recursive(*node_id, workspace, &mut function.nodes)
            }
            _ => todo!(),
        }
    }
}

fn lower_node_recursive(
    node_id: NodeId,
    workspace: &Workspace,
    nodes: &mut SlotMap<NodeId, Box<dyn EffectfulNode>>,
) {
    let node = mem::replace(&mut nodes[node_id], Box::new(Placeholder {}));
    let expanded = node.lowering_expand(node_id, workspace, nodes);
    if !expanded {
        // lowering couldn't be done, to put the original node back
        nodes[node_id] = node;
    };

    // try to expand its children
    for dependency in nodes[node_id].dependencies() {
        lower_node_recursive(dependency, workspace, nodes);
    }
}
