//! The `ask` command and `of` reporter.

use crate::hir::{ClosureType, Expr, ExprKind, HirToMirFnBuilder, NlAbstractTy, Program};
use crate::mir;

#[derive(Debug)]
pub enum AskRecipient {
    AllTurtles,
    AllPatches,
    TurtleAgentset(Box<ExprKind>),
    PatchAgentset(Box<ExprKind>),
    SingleTurtle(Box<ExprKind>),
    SinglePatch(Box<ExprKind>),
    Any(Box<ExprKind>),
}

impl AskRecipient {
    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        match self {
            AskRecipient::AllTurtles | AskRecipient::AllPatches => {}
            AskRecipient::TurtleAgentset(rec) => visitor(rec),
            AskRecipient::PatchAgentset(rec) => visitor(rec),
            AskRecipient::SingleTurtle(rec) => visitor(rec),
            AskRecipient::SinglePatch(rec) => visitor(rec),
            AskRecipient::Any(rec) => visitor(rec),
        }
    }
}

#[derive(Debug)]
pub struct Ask {
    /// The agents being asked.
    pub recipients: AskRecipient,
    /// The closure representing the commands to run for each recipient.
    pub body: Box<ExprKind>,
}

impl Expr for Ask {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        self.recipients.visit_children(&mut visitor);
        visitor(&self.body);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for Ask")
    }
}

#[derive(Debug)]
pub struct Of {
    /// The recipients to ask.
    pub recipients: AskRecipient,
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
        self.recipients.visit_children(&mut visitor);
        visitor(&self.body);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for Of")
    }
}
