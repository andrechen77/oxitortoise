//! The `distancexy` reporter.

use std::fmt;

use pretty_print::PrettyPrinter;

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy},
    mir,
};

#[derive(Debug, Clone)]
pub struct Distancexy {
    pub workspace: Box<ExprKind>,
    pub agent: Box<ExprKind>,
    pub x: Box<ExprKind>,
    pub y: Box<ExprKind>,
}

impl Expr for Distancexy {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Float
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.agent);
        visitor(&self.x);
        visitor(&self.y);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
        visitor(self.agent.as_mut());
        visitor(self.x.as_mut());
        visitor(self.y.as_mut());
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder) -> Option<mir::Place> {
        todo!("TODO(mvp) write MIR execution for Distancexy")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let Distancexy { workspace, agent, x, y } = self;
        p.add_fn_call("distancexy", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| agent.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| x.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| y.pretty_print(p, names))?;
            Ok(())
        })
    }
}
