//! The `create-turtles` command and friends.

use std::fmt::{self, Write};

use pretty_print::PrettyPrinter;

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy, TurtleBreedId},
    mir,
};

#[derive(Debug, Clone)]
pub struct CreateTurtles {
    pub workspace: Box<ExprKind>,
    pub rng: Box<ExprKind>,
    /// The breed of turtles to create.
    pub breed: TurtleBreedId,
    /// The number of turtles to create.
    pub num_turtles: Box<ExprKind>,
    /// The closure representing the commands to run for each created turtle.
    pub body: Box<ExprKind>,
}

impl Expr for CreateTurtles {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.rng);
        visitor(&self.num_turtles);
        visitor(&self.body);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
        visitor(self.rng.as_mut());
        visitor(self.num_turtles.as_mut());
        visitor(self.body.as_mut());
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for CreateTurtles")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let CreateTurtles { workspace, rng, breed, num_turtles, body } = self;
        p.add_fn_call("create_turtles", |p| {
            p.add_fn_arg_with(|p| {
                if let Some(b) = names.turtle_breeds().get(breed) {
                    write!(p, "{}#{}", breed, b.name)
                } else {
                    write!(p, "{:?}", breed)
                }
            })?;
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| rng.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| num_turtles.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| body.pretty_print(p, names))?;
            Ok(())
        })
    }
}
