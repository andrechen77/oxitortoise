//! Nodes for representing agentsets.

use crate::{
    hir::{Expr, NlAbstractTy, Program, build_mir::HirToMirFnBuilder},
    mir,
};

#[derive(Debug)]
pub enum Agentset {
    AllTurtles,
    AllPatches,
    // TODO(mvp) add links
}

impl Expr for Agentset {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        let typ = match self {
            Agentset::AllTurtles => NlAbstractTy::Turtle,
            Agentset::AllPatches => NlAbstractTy::Patch,
        };
        NlAbstractTy::Agentset { agent_type: Box::new(typ) }
    }

    fn visit_children(&self, _visitor: impl FnMut(&crate::hir::ExprKind)) {
        // nothing to do lolz
        match self {
            Agentset::AllTurtles => {}
            Agentset::AllPatches => {}
        }
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR code to generate a value representing the agentset")
    }
}
