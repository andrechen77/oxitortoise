//! Nodes for primitives relating purely to the tick counter.

use std::fmt;

use pretty_print::PrettyPrinter;

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy},
    mir,
};

#[derive(Debug, Clone)]
pub struct ResetTicks {
    pub workspace: Box<ExprKind>,
}

impl Expr for ResetTicks {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        todo!("TODO(mvp) write MIR execution for ResetTicks")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let ResetTicks { workspace } = self;
        p.add_fn_call("reset_ticks", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            Ok(())
        })
    }
}

#[derive(Debug, Clone)]
pub struct AdvanceTick {
    pub workspace: Box<ExprKind>,
}

impl Expr for AdvanceTick {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        todo!("TODO(mvp) write MIR execution for AdvanceTick")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let AdvanceTick { workspace } = self;
        p.add_fn_call("advance_tick", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            Ok(())
        })
    }
}

#[derive(Debug, Clone)]
pub struct GetTick {
    pub workspace: Box<ExprKind>,
}

impl Expr for GetTick {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Float
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        todo!("TODO(mvp) write MIR execution for GetTick")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let GetTick { workspace } = self;
        p.add_fn_call("get_tick", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            Ok(())
        })
    }
}
