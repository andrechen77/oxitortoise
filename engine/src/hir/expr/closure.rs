//! Nodes to represent closures.

use std::collections::BTreeMap;
use std::fmt::{self, Write};

use pretty_print::PrettyPrinter;

use crate::hir::{
    ClosureType, Expr, ExprKind, HirToMirFnBuilder, LocalDecl, LocalId, NameContext, NlAbstractTy,
};
use crate::mir;

#[derive(Debug, Clone)]
pub struct Closure {
    /// All the local variables that are captured by the closure.
    pub captures: Vec<LocalId>,
    pub parameters: BTreeMap<LocalId, LocalDecl>,
    /// The body of the closure. This is the part of the closure with deferred
    /// evaluation.
    pub body: Box<ExprKind>,
}

impl Expr for Closure {
    fn output_type(&self, names: NameContext) -> NlAbstractTy {
        let return_ty = self.body.output_type(names);

        NlAbstractTy::Closure(ClosureType {
            arg_tys: self.parameters.values().map(|decl| decl.ty.clone()).collect(),
            return_ty: Box::new(return_ty),
        })
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.body);
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        visitor(self.body.as_mut());
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for Closure")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let Closure { captures, parameters, body } = self;
        p.add_fn_call("closure", |p| {
            p.add_fn_arg_with(|p| {
                write!(p, "[")?;
                for (i, c) in captures.iter().enumerate() {
                    if i > 0 {
                        write!(p, ", ")?;
                    }
                    write!(p, "{:?}", c)?;
                }
                write!(p, "]")
            })?;
            p.add_fn_arg_with(|p| {
                write!(p, "(")?;
                p.indented(|p| {
                    for (local_id, decl) in parameters {
                        p.line()?;
                        write!(
                            p,
                            "{:?} {}: {},",
                            local_id,
                            decl.debug_name.as_ref().map_or("", |n| n.as_ref()),
                            decl.ty
                        )?;
                    }
                    Ok(())
                })?;
                p.line()?;
                write!(p, ")")
            })?;
            p.add_fn_arg_with(|p| body.pretty_print(p, names.with_locals(parameters)))?;
            Ok(())
        })
    }
}
