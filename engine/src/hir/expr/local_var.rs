//! Nodes for getting and setting local variables.

use std::fmt::{self, Write};

use pretty_print::PrettyPrinter;
use tracing::warn;

use crate::{
    hir::{
        Expr, ExprKind, HirToMirFnBuilder, LocalId, NameContext, NlAbstractTy, NlAbstractTyAtom,
        build_mir::translate_expr,
    },
    mir,
};

#[derive(Debug, Clone)]
pub struct GetLocalVar {
    pub local_id: LocalId,
}

impl Expr for GetLocalVar {
    fn output_type(&self, names: NameContext) -> NlAbstractTy {
        names.lookup_local_var(self.local_id).unwrap().ty.clone()
    }

    fn visit_children<'a>(&'a self, _visitor: impl FnMut(&'a ExprKind)) {
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

impl GetLocalVar {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let Some(place) = builder.local_translator.locals.get(&self.local_id) else {
            panic!(
                "local variable {:?} not found in translator: {:?}",
                self.local_id, builder.local_translator.locals
            );
        };
        if place.projections.is_empty() {
            let local = place.unwrap_local();
            if !builder.mir.is_init(local) {
                warn!("getting uninitialized local: {:?}", local);
            }
            Some(local)
        } else {
            Some(builder.mir.clone_to_new(place.clone()))
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetLocalVar {
    pub local_id: LocalId,
    pub value: Box<ExprKind>,
}

impl Expr for SetLocalVar {
    fn output_type(&self, _names: NameContext) -> NlAbstractTy {
        NlAbstractTyAtom::Unit.into()
    }

    fn visit_children<'a>(&'a self, mut visitor: impl FnMut(&'a ExprKind)) {
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

impl SetLocalVar {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let value = translate_expr(builder, &self.value)?;
        let dst = builder.local_translator.locals.get(&self.local_id).unwrap().unwrap_local();

        if builder.mir.is_init(dst) {
            builder.mir.move_to_init(dst.place(), value);
        } else {
            builder.mir.add_operation_with_dst(
                dst.place(),
                mir::Operation::Operand(mir::PlaceOperand::Move(value)),
            );
        }

        Some(builder.mir.unit_local())
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
