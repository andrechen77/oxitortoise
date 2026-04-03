//! The `clear-all` command and friends.

use std::fmt;

use pretty_print::PrettyPrinter;

use crate::hir::{Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy};
use crate::mir;

#[derive(Debug, Clone)]
pub struct ClearAll {
    pub workspace: Box<ExprKind>,
}

impl Expr for ClearAll {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder) -> Option<mir::Place> {
        todo!("TODO(mvp) write MIR execution for ClearAll")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let ClearAll { workspace } = self;
        p.add_fn_call("clear_all", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            Ok(())
        })
    }
}
