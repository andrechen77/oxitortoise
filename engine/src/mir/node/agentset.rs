use derive_more::derive::Display;
use lir::smallvec::SmallVec;
use slotmap::SlotMap;

use crate::mir::{EffectfulNode, LocalDeclaration, LocalId, NodeId, build_lir::LirInsnBuilder};

#[derive(Debug, Display)]
enum AgentSet {
    #[display("AllTurtles")]
    AllTurtles,
    #[display("AllPatches")]
    AllPatches,
    #[display("TurtleAgentset")]
    TurtleAgentset(NodeId),
    #[display("PatchAgentset")]
    PatchAgentset(NodeId),
    #[display("SingleTurtle")]
    SingleTurtle(NodeId),
    #[display("SinglePatch")]
    SinglePatch(NodeId),
    // TODO add links
}

impl EffectfulNode for AgentSet {
    fn has_side_effects(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        match self {
            AgentSet::AllTurtles => vec![],
            AgentSet::AllPatches => vec![],
            AgentSet::TurtleAgentset(id) => vec![*id],
            AgentSet::PatchAgentset(id) => vec![*id],
            AgentSet::SingleTurtle(id) => vec![*id],
            AgentSet::SinglePatch(id) => vec![*id],
        }
    }

    fn output_type(
        &self,
        _workspace: &crate::workspace::Workspace,
        _nodes: &slotmap::SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<crate::sim::value::NetlogoInternalType> {
        todo!("match on internal variant and deduce type")
    }

    fn write_lir_execution(
        &self,
        my_node_id: NodeId,
        nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        lir_builder: &mut crate::mir::build_lir::LirInsnBuilder,
    ) -> Result<(), ()> {
        todo!()
    }
}
