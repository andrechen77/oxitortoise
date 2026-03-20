//! The `distancexy` reporter.

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, NlAbstractTy, Program},
    mir,
};

#[derive(Debug)]
pub struct Distancexy {
    pub agent: Box<ExprKind>,
    pub x: Box<ExprKind>,
    pub y: Box<ExprKind>,
}

impl Expr for Distancexy {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Float
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.agent);
        visitor(&self.x);
        visitor(&self.y);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for Distancexy")
    }
}

