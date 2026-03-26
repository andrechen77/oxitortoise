//! Nodes for commands/reporters that interact with the RNG.

use std::fmt;

use pretty_print::PrettyPrinter;

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy},
    mir,
};

#[derive(Debug, Clone)]
pub struct RandomInt {
    pub rng: Box<ExprKind>,
    pub bound: Box<ExprKind>,
}

impl Expr for RandomInt {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        // Despite the name, the current HIR uses `Float` as the abstract output type.
        NlAbstractTy::Float
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.rng);
        visitor(&self.bound);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.rng.as_mut());
        visitor(self.bound.as_mut());
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for RandomInt")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let RandomInt { rng, bound } = self;
        p.add_fn_call("random_int", |p| {
            p.add_fn_arg_with(|p| rng.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| bound.pretty_print(p, names))?;
            Ok(())
        })
    }
}
