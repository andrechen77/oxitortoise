//! Nodes for getting and setting local variables.

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, LocalId, NlAbstractTy, Program},
    mir,
};

#[derive(Debug)]
pub struct GetLocalVar {
    pub local_id: LocalId,
}

impl Expr for GetLocalVar {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        todo!("TODO(mvp) GetLocalVar output type inference")
    }

    fn visit_children(&self, _visitor: impl FnMut(&ExprKind)) {
        // no children
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for GetLocalVar")
    }
}

#[derive(Debug)]
pub struct SetLocalVar {
    pub local_id: LocalId,
    pub value: Box<ExprKind>,
}

impl Expr for SetLocalVar {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.value);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for SetLocalVar")
    }
}

