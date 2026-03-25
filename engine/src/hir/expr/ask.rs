//! The `ask` command and `of` reporter.

use std::fmt;

use pretty_print::PrettyPrinter;

use crate::hir::format::NameContext;
use crate::hir::{ClosureType, Expr, ExprKind, HirToMirFnBuilder, NlAbstractTy, Program};
use crate::mir;

#[derive(Debug, Clone)]
pub struct Ask {
    pub workspace: Box<ExprKind>,
    pub rng: Box<ExprKind>,
    /// The agents being asked.
    pub recipients: Box<ExprKind>,
    /// The closure representing the commands to run for each recipient.
    pub body: Box<ExprKind>,
}

impl Expr for Ask {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.rng);
        visitor(&self.recipients);
        visitor(&self.body);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for Ask")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let Ask { workspace, rng, recipients, body } = self;
        p.add_fn_call("ask", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| rng.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| recipients.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| body.pretty_print(p, names))?;
            Ok(())
        })
    }
}

#[derive(Debug, Clone)]
pub struct Of {
    pub workspace: Box<ExprKind>,
    pub rng: Box<ExprKind>,
    /// The recipients to ask.
    pub recipients: Box<ExprKind>,
    /// The closure representing the reporter to run for each recipient.
    pub body: Box<ExprKind>,
}

impl Expr for Of {
    fn output_type(&self, program: &Program) -> NlAbstractTy {
        let body_ty = self.body.output_type(program);
        let NlAbstractTy::Closure(ClosureType { return_ty, .. }) = body_ty else {
            panic!("Of body must have closure type, got: {:?}", body_ty);
        };
        *return_ty
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.rng);
        visitor(&self.recipients);
        visitor(&self.body);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for Of")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let Of { workspace, rng, recipients, body } = self;
        p.add_fn_call("of", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| rng.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| recipients.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| body.pretty_print(p, names))?;
            Ok(())
        })
    }
}
