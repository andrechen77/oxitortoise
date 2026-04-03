//! Nodes for commands/reporters that interact with colors.

use std::fmt;

use pretty_print::PrettyPrinter;

use crate::hir::NameContext;
use crate::hir::{Expr, ExprKind, HirToMirFnBuilder, NlAbstractTy};
use crate::mir;

/// NetLogo `scale-color`.
#[derive(Debug, Clone)]
pub struct ScaleColor {
    pub color: Box<ExprKind>,
    pub number: Box<ExprKind>,
    pub range1: Box<ExprKind>,
    pub range2: Box<ExprKind>,
}

impl Expr for ScaleColor {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Color
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.color);
        visitor(&self.number);
        visitor(&self.range1);
        visitor(&self.range2);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.color.as_mut());
        visitor(self.number.as_mut());
        visitor(self.range1.as_mut());
        visitor(self.range2.as_mut());
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder) -> Option<mir::Place> {
        todo!("TODO(mvp) write MIR execution for ScaleColor")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let ScaleColor { color, number, range1, range2 } = self;
        p.add_fn_call("scale_color", |p| {
            p.add_fn_arg_with(|p| color.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| number.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| range1.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| range2.pretty_print(p, names))?;
            Ok(())
        })
    }
}
