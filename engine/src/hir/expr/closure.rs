//! Nodes to represent closures.

use crate::hir::{
    ClosureType, Expr, ExprKind, HirToMirFnBuilder, LocalDecl, LocalId, NlAbstractTy, Program,
};
use crate::mir;

#[derive(Debug)]
pub struct Closure {
    /// All the local variables that are captured by the closure.
    pub captures: Vec<LocalId>,
    pub parameters: Vec<(LocalId, LocalDecl)>,
    /// The body of the closure. This is the part of the closure with deferred
    /// evaluation.
    pub body: Box<ExprKind>,
}

impl Expr for Closure {
    fn output_type(&self, program: &Program) -> NlAbstractTy {
        let return_ty = self.body.output_type(program);

        NlAbstractTy::Closure(ClosureType {
            arg_tys: self.parameters.iter().map(|(_, decl)| decl.ty.clone()).collect(),
            return_ty: Box::new(return_ty),
        })
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.body);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for Closure")
    }
}
