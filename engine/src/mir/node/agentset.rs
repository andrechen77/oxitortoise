//! Nodes for representing agentsets.

use pretty_print::PrettyPrinter;

use crate::{
    exec::jit::InstallLir,
    mir::{
        FunctionId, MirTy,
        NlAbstractTy::{self, Patch, Turtle},
        Node, NodeId, Program, WriteLirError,
        build_lir::LirInsnBuilder,
    },
};

#[derive(Debug)]
pub enum Agentset {
    AllTurtles,
    AllPatches,
    // TODO(mvp) add links
}

impl Node for Agentset {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<(&'static str, NodeId)> {
        vec![]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        let typ = match self {
            Agentset::AllTurtles => Turtle,
            Agentset::AllPatches => Patch,
        };
        NlAbstractTy::Agentset { agent_type: Box::new(typ) }.into()
    }

    fn pretty_print(&self, _program: &Program, mut out: impl std::fmt::Write) -> std::fmt::Result {
        let name = match self {
            Agentset::AllTurtles => "AllTurtles",
            Agentset::AllPatches => "AllPatches",
        };
        PrettyPrinter::new(&mut out).add_struct(name, |_| Ok(()))
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
