//! Nodes for representing agentsets.

use derive_more::derive::Display;

use crate::{
    exec::jit::InstallLir,
    mir::{
        FunctionId, MirTy,
        NlAbstractTy::{self, Patch, Turtle},
        Node, NodeId, Program, WriteLirError,
        build_lir::LirInsnBuilder,
    },
};

#[derive(Debug, Display)]
pub enum Agentset {
    #[display("AllTurtles")]
    AllTurtles,
    #[display("AllPatches")]
    AllPatches,
    // TODO(mvp) add links
}

impl Node for Agentset {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        match self {
            Agentset::AllTurtles => vec![],
            Agentset::AllPatches => vec![],
        }
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        let typ = match self {
            Agentset::AllTurtles => Turtle,
            Agentset::AllPatches => Patch,
        };
        NlAbstractTy::Agentset { agent_type: Box::new(typ) }.into()
    }

    fn write_lir_execution<I: InstallLir>(
        &self,
        _program: &Program,
        _my_node_id: NodeId,
        _lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        todo!("TODO(mvp) write LIR code to generate a value representing the agentset")
    }
}
