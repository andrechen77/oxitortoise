//! Nodes for commands/reporters that interact with colors.

use crate::hir::{Expr, ExprKind, HirToMirFnBuilder, NlAbstractTy, Program};
use crate::mir;

/// NetLogo `scale-color`.
#[derive(Debug)]
pub struct ScaleColor {
    pub color: Box<ExprKind>,
    pub number: Box<ExprKind>,
    pub range1: Box<ExprKind>,
    pub range2: Box<ExprKind>,
}

impl Expr for ScaleColor {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Color
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.color);
        visitor(&self.number);
        visitor(&self.range1);
        visitor(&self.range2);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for ScaleColor")
    }
}

