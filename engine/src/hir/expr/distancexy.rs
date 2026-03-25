//! The `distancexy` reporter.

use std::fmt;

use pretty_print::PrettyPrinter;

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, NlAbstractTy, Program},
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
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Float
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.agent);
        visitor(&self.x);
        visitor(&self.y);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for Distancexy")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        program: &Program,
    ) -> fmt::Result {
        let Distancexy { workspace, agent, x, y } = self;
        p.add_fn_call("distancexy", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, program))?;
            p.add_fn_arg_with(|p| agent.pretty_print(p, program))?;
            p.add_fn_arg_with(|p| x.pretty_print(p, program))?;
            p.add_fn_arg_with(|p| y.pretty_print(p, program))?;
            Ok(())
        })
    }
}
