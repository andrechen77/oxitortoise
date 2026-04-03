//! The `diffuse` command.

use std::fmt;

use pretty_print::PrettyPrinter;

use crate::mir;
use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy},
    sim::patch::PatchVarDesc,
};

#[derive(Debug, Clone)]
pub struct Diffuse {
    pub workspace: Box<ExprKind>,
    /// The patch variable to diffuse.
    pub variable: PatchVarDesc,
    /// The amount to diffuse.
    pub amt: Box<ExprKind>,
}

impl Expr for Diffuse {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
        visitor(&self.amt);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
        visitor(self.amt.as_mut());
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        todo!("TODO(mvp) write MIR execution for Diffuse")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let Diffuse { workspace, variable, amt } = self;
        p.add_fn_call("diffuse", |p| {
            p.add_fn_arg_with(|p| variable.pretty_print(p, names.custom_patch_vars()))?;
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            p.add_fn_arg_with(|p| amt.pretty_print(p, names))?;
            Ok(())
        })
    }
}
