//! The `diffuse` command.

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, NlAbstractTy, Program},
    sim::patch::PatchVarDesc,
};
use crate::mir;

#[derive(Debug)]
pub struct Diffuse {
    /// The patch variable to diffuse.
    pub variable: PatchVarDesc,
    /// The amount to diffuse.
    pub amt: Box<ExprKind>,
}

impl Expr for Diffuse {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.amt);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for Diffuse")
    }
}

