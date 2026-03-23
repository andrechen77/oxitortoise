//! Nodes that represent constant/literal values.

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, NlAbstractTy, Program},
    mir,
    sim::value::UnpackedAny,
};

#[derive(Debug)]
pub struct Constant {
    pub value: Option<UnpackedAny>,
}

impl Expr for Constant {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        let Some(value) = &self.value else {
            return NlAbstractTy::Unit;
        };
        match value {
            UnpackedAny::Float(_) => NlAbstractTy::Float,
            UnpackedAny::Bool(_) => NlAbstractTy::Boolean,
            UnpackedAny::Nobody => NlAbstractTy::Nobody,
            _ => todo!("TODO(mvp) include all other Constant variants"),
        }
    }

    fn visit_children(&self, _visitor: impl FnMut(&ExprKind)) {
        // no children
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for Constant")
    }
}

#[derive(Debug)]
pub struct ListLiteral {
    pub items: Vec<Box<ExprKind>>,
}

impl Expr for ListLiteral {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        // Until we have element-type inference for list literals.
        NlAbstractTy::List { element_ty: Box::new(NlAbstractTy::Top) }
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        for item in &self.items {
            visitor(item);
        }
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for ListLiteral")
    }
}
