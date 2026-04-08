//! Nodes for primitives that operate on lists and agentsets.

use std::fmt;

use pretty_print::PrettyPrinter;

use crate::hir::{Expr, ExprKind, NameContext, NlAbstractTy};

#[derive(Debug, Clone)]
pub struct OneOf {
    pub rng: Box<ExprKind>,
    pub operand: Box<ExprKind>,
}

impl Expr for OneOf {
    fn output_type(&self, names: NameContext) -> NlAbstractTy {
        let operand_ty = self.operand.output_type(names);
        match operand_ty {
            NlAbstractTy::Agentset { agent_type } => *agent_type,
            NlAbstractTy::List { element_ty } => *element_ty,
            x => todo!("TODO(mvp) OneOf unsupported operand type: {:?}", x),
        }
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.rng);
        visitor(&self.operand);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.rng.as_mut());
        visitor(self.operand.as_mut());
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let OneOf { rng, operand } = self;
        p.add_fn_call("one_of", |p| {
            p.add_fn_arg_with(|p| rng.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| operand.pretty_print(p, names))?;
            Ok(())
        })
    }
}
