//! The `create-turtles` command and friends.

use crate::hir::{Expr, ExprKind, HirToMirFnBuilder, NlAbstractTy, Program};
use crate::sim::turtle::BreedId;
use crate::mir;

#[derive(Debug)]
pub struct CreateTurtles {
    /// The breed of turtles to create.
    pub breed: BreedId,
    /// The number of turtles to create.
    pub num_turtles: Box<ExprKind>,
    /// The closure representing the commands to run for each created turtle.
    pub body: Box<ExprKind>,
}

impl Expr for CreateTurtles {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.num_turtles);
        visitor(&self.body);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for CreateTurtles")
    }
}

