use derive_more::derive::Display;

use crate::mir::{
    EffectfulNode, Function, MirTy, NlAbstractTy,
    NlAbstractTy::{Patch, Turtle},
    NodeId, Nodes, Program, WriteLirError,
};

#[derive(Debug, Display)]
pub enum Agentset {
    #[display("AllTurtles")]
    AllTurtles,
    #[display("AllPatches")]
    AllPatches,
    // TODO(mvp) add links
}

impl EffectfulNode for Agentset {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        match self {
            Agentset::AllTurtles => vec![],
            Agentset::AllPatches => vec![],
        }
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        let typ = match self {
            Agentset::AllTurtles => Turtle,
            Agentset::AllPatches => Patch,
        };
        MirTy::Abstract(NlAbstractTy::Agentset { agent_type: Box::new(typ) })
    }

    fn write_lir_execution(
        &self,
        _program: &Program,
        _function: &Function,
        _nodes: &Nodes,
        _my_node_id: NodeId,
        _lir_builder: &mut crate::mir::build_lir::LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        todo!("TODO(mvp) write LIR code to generate a value representing the agentset")
    }
}
