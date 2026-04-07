//! The `clear-all` command and friends.

use std::fmt;

use pretty_print::PrettyPrinter;

use crate::hir::{Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy};
use crate::mir;

#[derive(Debug, Clone)]
pub struct ClearAll {
    pub workspace: Box<ExprKind>,
}

impl Expr for ClearAll {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.workspace);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
    }

    fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let workspace_local = self.workspace.write_mir_execution(builder)?;

        let operation = mir::Operation::CallHostFunction {
            function: &clear_all::FN_INFO,
            args: vec![mir::PlaceOperand::Copy(workspace_local.place())],
        };
        Some(builder.mir.add_operation(None, operation))
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let ClearAll { workspace } = self;
        p.add_fn_call("clear_all", |p| {
            p.add_fn_arg_with(|p| workspace.pretty_print(p, names))?;
            Ok(())
        })
    }
}

mod clear_all {
    use crate::{mir::HostFunctionInfo, util::reflection::Reflect, workspace::Workspace};

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "clear_all",
        parameter_types: &[<&mut Workspace>::TYPE],
        return_type: <()>::TYPE,
    };

    pub fn call(workspace: &mut Workspace) {
        todo!()
    }
}
