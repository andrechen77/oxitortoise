//! The `clear-all` command and friends.

use crate::hir::{Expr, ExprKind, HirToMirFnBuilder, NlAbstractTy, Program};
use crate::mir;

#[derive(Debug)]
pub struct ClearAll;

impl Expr for ClearAll {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, _visitor: impl FnMut(&ExprKind)) {
        // no children
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for ClearAll")
    }
}

