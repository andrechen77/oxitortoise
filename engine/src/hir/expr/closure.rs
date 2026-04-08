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

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let Closure { captures, parameters, body } = self;
        p.add_struct("closure", |p| {
            p.add_field_with("captures", |p| {
                p.add_list(captures.iter(), |p, capture| {
                    let name = names
                        .lookup_local_var(*capture)
                        .map(|decl| decl.debug_name.as_ref())
                        .unwrap_or("?");
                    write!(p, "{}#{}", capture.0, name)
                })
            })?;
            p.add_field_with("parameters", |p| {
                p.add_list(parameters.iter(), |p, (local_id, decl)| {
                    write!(p, "{}#{}: {}", local_id.0, decl.debug_name, decl.ty)
                })
            })?;
            p.add_field_with("body", |p| body.pretty_print(p, names.with_locals(parameters)))?;
            Ok(())
        })
    }
}

impl Closure {
    pub fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        /*
        there are a couple of scenarios to consider for the closure

        no captures: the env pointer can be any dangling pointer, including null.
        the call function does not use the env pointer and the drop function does nothing.

        captures without escape: the env pointer is a non-owning reference that points to the stack
        frame, and the call function uses stack frame offsets to access the
        captured variables.

        captures with escape: the env pointers is a owning pointer to an anonymous
        struct that contains captured variables. the anonymous struct is initialized
        by cloning all captured variables into the anonymous struct. to ensure
        that modifications to the captured variables are visible to the creator
        of the closure, captured variables that are modified must be shared RC
        pointers to the actual value
         */
        todo!("TODO(mvp) write MIR execution for Closure")
    }
}
