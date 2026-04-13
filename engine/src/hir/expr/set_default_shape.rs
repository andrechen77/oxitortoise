//! The `set-default-shape` command.

use std::fmt::{self, Write};

use pretty_print::PrettyPrinter;

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy, TurtleBreedId},
    mir,
};

#[derive(Debug, Clone)]
pub struct SetDefaultShape {
    pub workspace: Box<ExprKind>,
    /// The breed to set the default shape for.
    pub breed: TurtleBreedId,
    /// The shape to set.
    pub shape: Box<ExprKind>,
}

impl Expr for SetDefaultShape {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.shape);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
        visitor(self.shape.as_mut());
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let SetDefaultShape { workspace, breed, shape } = self;
        p.add_fn_call("set_default_shape", |p| {
            p.add_fn_arg_with(|p| {
                if let Some(b) = names.turtle_breeds().get(breed) {
                    write!(p, "{}#{}", breed, b.name)
                } else {
                    write!(p, "{:?}", breed)
                }
            })?;
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| shape.pretty_print(p, names))?;
            Ok(())
        })
    }
}

impl SetDefaultShape {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        // TODO(mvp) write LIR code to set the default shape
        Some(builder.mir.unit_local())
    }
}
