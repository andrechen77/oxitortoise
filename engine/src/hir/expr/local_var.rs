//! Nodes for getting and setting local variables.

use std::fmt::{self, Write};

use pretty_print::PrettyPrinter;

use crate::hir::{Expr, ExprKind, LocalId, NameContext, NlAbstractTy};

#[derive(Debug, Clone)]
pub struct GetLocalVar {
    pub local_id: LocalId,
}

impl Expr for GetLocalVar {
    fn output_type(&self, names: NameContext) -> NlAbstractTy {
        names.lookup_local_var(self.local_id).unwrap().ty.clone()
    }

    fn visit_children(&self, _visitor: impl FnMut(&ExprKind)) {
        // no children
    }

    fn visit_children_mut(&mut self, _visitor: impl FnMut(&mut ExprKind)) {}

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

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.value.as_mut());
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
    let name = names.lookup_local_var(local_id).map(|decl| decl.debug_name.as_ref()).unwrap_or("?");
    write!(p, "{}#{}", local_id.0, name)
}
