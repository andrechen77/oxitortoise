//! The `clear-all` command and friends.

use std::fmt;

use pretty_print::PrettyPrinter;
use reflection::mir;

use crate::hir::build_mir::translate_expr;
use crate::hir::ty::NlAbstractTyAtom;
use crate::hir::{Expr, ExprKind, HirToMirFnBuilder, NameContext, NlAbstractTy};

#[derive(Debug, Clone)]
pub struct ClearAll {
    pub workspace: Box<ExprKind>,
}

impl Expr for ClearAll {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTyAtom::Unit.into()
    }

    fn visit_children<'a>(&'a self, mut visitor: impl FnMut(&'a ExprKind)) {
        visitor(&self.workspace);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.workspace.as_mut());
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

impl ClearAll {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let workspace_local = translate_expr(builder, &self.workspace)?;

        let operation = mir::Operation::CallHostFunction {
            function: &clear_all::FN_INFO,
            args: vec![mir::PlaceOperand::Direct(workspace_local.place())],
        };
        Some(builder.mir.add_operation(None, operation))
    }
}

mod clear_all {
    use crate::workspace::Workspace;
    use reflection::{Reflect, mir::HostFunctionInfo};

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "clear_all",
        parameter_types: &[<&mut Workspace>::STATIC_TYPE],
        return_type: <()>::STATIC_TYPE,
        link_name: "clear_all",
        link_addr: call as *const u8,
    };

    pub fn call(_workspace: &mut Workspace) {
        todo!()
    }
}
