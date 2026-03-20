//! Nodes for primitives relating purely to the tick counter.

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, NlAbstractTy, Program},
    mir,
};

#[derive(Debug)]
pub struct ResetTicks;

impl Expr for ResetTicks {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, _visitor: impl FnMut(&ExprKind)) {
        // no children
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for ResetTicks")
    }
}

#[derive(Debug)]
pub struct AdvanceTick;

impl Expr for AdvanceTick {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, _visitor: impl FnMut(&ExprKind)) {
        // no children
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for AdvanceTick")
    }
}

#[derive(Debug)]
pub struct GetTick;

impl Expr for GetTick {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Float
    }

    fn visit_children(&self, _visitor: impl FnMut(&ExprKind)) {
        // no children
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for GetTick")
    }
}

