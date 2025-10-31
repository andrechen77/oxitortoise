use derive_more::derive::Display;
use slotmap::SlotMap;

use crate::mir::{
    EffectfulNode, Function, MirType, NetlogoAbstractType,
    NetlogoAbstractType::{Patch, Turtle},
    NodeId, Nodes, Program, WriteLirError,
};

#[derive(Debug, Display)]
pub enum Agentset {
    #[display("AllTurtles")]
    AllTurtles,
    #[display("AllPatches")]
    AllPatches,
    // #[display("TurtleAgentset")]
    // TurtleAgentset(NodeId),
    // #[display("PatchAgentset")]
    // PatchAgentset(NodeId),
    // #[display("SingleTurtle")]
    // SingleTurtle(NodeId),
    // #[display("SinglePatch")]
    // SinglePatch(NodeId),
    // TODO add links
}

impl EffectfulNode for Agentset {
    fn has_side_effects(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        match self {
            Agentset::AllTurtles => vec![],
            Agentset::AllPatches => vec![],
            // Agentset::TurtleAgentset(id) => vec![*id],
            // Agentset::PatchAgentset(id) => vec![*id],
            // Agentset::SingleTurtle(id) => vec![*id],
            // Agentset::SinglePatch(id) => vec![*id],
        }
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirType {
        let typ = match self {
            Agentset::AllTurtles => Turtle,
            Agentset::AllPatches => Patch,
        };
        MirType::Abstract(NetlogoAbstractType::Agentset { agent_type: Box::new(typ) })
    }

    fn write_lir_execution(
        &self,
        _my_node_id: NodeId,
        _nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _lir_builder: &mut crate::mir::build_lir::LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        todo!()
    }
}
