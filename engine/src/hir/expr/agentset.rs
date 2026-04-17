//! Nodes for representing agentsets.

use std::{
    fmt::{self, Write},
    sync::Arc,
};

use pretty_print::PrettyPrinter;

use crate::hir::{Expr, ExprKind, NameContext, NlAbstractTy, ty::NlAbstractTyAtom};

#[derive(Debug, Clone)]
pub enum Agentset {
    AllTurtles,
    AllPatches,
    // TODO(mvp) add links
}

impl Expr for Agentset {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        let typ = match self {
            Agentset::AllTurtles => NlAbstractTyAtom::Turtle.into(),
            Agentset::AllPatches => NlAbstractTyAtom::Patch.into(),
        };
        NlAbstractTyAtom::Agentset { agent_type: Arc::new(typ) }.into()
    }

    fn visit_children<'a>(&'a self, _visitor: impl FnMut(&'a crate::hir::ExprKind)) {
        // nothing to do lolz
    }

    fn visit_children_mut(&mut self, _visitor: impl FnMut(&mut ExprKind)) {
        match self {
            Agentset::AllTurtles => {}
            Agentset::AllPatches => {}
        }
    }

    fn pretty_print<W: Write>(&self, p: &mut PrettyPrinter<W>, _names: NameContext) -> fmt::Result {
        match self {
            Agentset::AllTurtles => write!(p, "all_turtles()"),
            Agentset::AllPatches => write!(p, "all_patches()"),
        }
    }
}
