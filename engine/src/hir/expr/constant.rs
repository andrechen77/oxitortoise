//! Nodes that represent constant/literal values.

use std::fmt::{self, Write};

use pretty_print::PrettyPrinter;

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy},
    mir,
    sim::value::UnpackedAny,
};

#[derive(Debug, Clone)]
pub struct Constant {
    pub value: Option<UnpackedAny>,
}

impl Expr for Constant {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
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

    fn visit_children_mut(&mut self, _visitor: impl FnMut(&mut ExprKind)) {}

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder) -> Option<mir::Place> {
        todo!("TODO(mvp) write MIR execution for Constant")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        _names: NameContext,
    ) -> fmt::Result {
        p.add_fn_call("constant", |p| {
            p.add_fn_arg_with(|p| match &self.value {
                None => write!(p, "none"),
                Some(v) => write!(p, "{:?}", v),
            })?;
            Ok(())
        })
    }
}

#[derive(Debug, Clone)]
pub struct ListLiteral {
    pub items: Vec<Box<ExprKind>>,
}

impl Expr for ListLiteral {
    fn output_type(&self, names: NameContext) -> NlAbstractTy {
        let mut ty = NlAbstractTy::Bottom;
        for item in &self.items {
            ty = ty.join(item.output_type(names));
        }
        NlAbstractTy::List { element_ty: Box::new(ty) }
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        for item in &self.items {
            visitor(item);
        }
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        for item in &mut self.items {
            visitor(item.as_mut());
        }
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder) -> Option<mir::Place> {
        todo!("TODO(mvp) write MIR execution for ListLiteral")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        p.add_fn_call("list_literal", |p| {
            for item in &self.items {
                p.add_fn_arg_with(|p| item.pretty_print(p, names))?;
            }
            Ok(())
        })
    }
}
