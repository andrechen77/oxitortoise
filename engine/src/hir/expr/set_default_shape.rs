//! The `set-default-shape` command.

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, NlAbstractTy, Program},
    sim::turtle::BreedId,
};
use crate::mir;

#[derive(Debug)]
pub struct SetDefaultShape {
    /// The breed to set the default shape for.
    pub breed: BreedId,
    /// The shape to set.
    pub shape: Box<ExprKind>,
}

impl Expr for SetDefaultShape {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.shape);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for SetDefaultShape")
    }
}

