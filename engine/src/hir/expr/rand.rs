//! Nodes for commands/reporters that interact with the RNG.

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, NlAbstractTy, Program},
    mir,
};

#[derive(Debug)]
pub struct RandomInt {
    pub bound: Box<ExprKind>,
}

impl Expr for RandomInt {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        // Despite the name, the current HIR uses `Float` as the abstract output type.
        NlAbstractTy::Float
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.bound);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for RandomInt")
    }
}

