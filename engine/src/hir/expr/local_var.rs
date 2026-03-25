//! Nodes for getting and setting local variables.

use std::fmt::{self, Write};

use pretty_print::PrettyPrinter;

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, LocalId, NameContext, NlAbstractTy},
    mir,
};

#[derive(Debug, Clone)]
pub struct GetLocalVar {
    pub local_id: LocalId,
}

impl Expr for GetLocalVar {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        todo!("TODO(mvp) GetLocalVar output type inference")
    }

    fn visit_children(&self, _visitor: impl FnMut(&ExprKind)) {
        // no children
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for GetLocalVar")
    }

    fn pretty_print<W: Write>(&self, p: &mut PrettyPrinter<W>, names: NameContext) -> fmt::Result {
        p.add_fn_call("get", |p| {
            p.add_fn_arg_with(|p| pretty_print_local(p, self.local_id, names))?;
            Ok(())
        })
    }
}

#[derive(Debug, Clone)]
pub struct SetLocalVar {
    pub local_id: LocalId,
    pub value: Box<ExprKind>,
}

impl Expr for SetLocalVar {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTy::Unit
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.value);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for SetLocalVar")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let SetLocalVar { local_id, value } = self;
        p.add_fn_call("set", |p| {
            p.add_fn_arg_with(|p| pretty_print_local(p, *local_id, names))?;
            p.add_fn_arg_with(|p| value.pretty_print(p, names))?;
            Ok(())
        })
    }
}

fn pretty_print_local(
    p: &mut PrettyPrinter<impl Write>,
    local_id: LocalId,
    names: NameContext,
) -> fmt::Result {
    let name =
        names.lookup_local_var(local_id).and_then(|decl| decl.debug_name.as_deref()).unwrap_or("?");
    write!(p, "{}#{}", local_id.0, name)
}
