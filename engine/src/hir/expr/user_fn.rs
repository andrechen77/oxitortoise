//! Nodes for a call to a user-defined function.

use std::fmt::{self, Write};

use pretty_print::PrettyPrinter;

use crate::{
    hir::{
        Expr, ExprKind, FunctionId, HirToMirFnBuilder, NameContext, NlAbstractTy,
        build_mir::translate_expr,
    },
};
use reflection::mir;

#[derive(Debug, Clone)]
pub struct CallUserFn {
    /// The function being called.
    pub target: FunctionId,
    /// The arguments to the function.
    pub args: Vec<Box<ExprKind>>,
}

impl Expr for CallUserFn {
    fn output_type(&self, names: NameContext) -> NlAbstractTy {
        names.functions()[&self.target].return_ty.clone()
    }

    fn visit_children<'a>(&'a self, mut visitor: impl FnMut(&'a ExprKind)) {
        for arg in &self.args {
            visitor(arg);
        }
    }

    fn visit_children_mut(&mut self, mut visitor: impl FnMut(&mut ExprKind)) {
        for arg in &mut self.args {
            visitor(arg.as_mut());
        }
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        names: NameContext,
    ) -> fmt::Result {
        let CallUserFn { target, args } = self;
        p.add_fn_call("call_user_fn", |p| {
            p.add_fn_arg_with(|p| {
                let label = &names.functions()[target].debug_name;
                write!(p, "{}#{}", target.0, label)
            })?;
            for arg in args {
                p.add_fn_arg_with(|p| arg.pretty_print(p, names))?;
            }
            Ok(())
        })
    }
}

impl CallUserFn {
    pub fn write_mir_execution(&self, builder: &mut HirToMirFnBuilder) -> Option<mir::LocalId> {
        let mut args = Vec::new();
        for arg in &self.args {
            let local_id = translate_expr(builder, arg)?;
            args.push(mir::PlaceOperand::Move(local_id));
        }
        let target_mir_id = builder.user_fn_translator[&self.target];

        let operation = mir::Operation::CallUserFunction { function: target_mir_id, args };

        Some(builder.mir.add_operation(None, operation))
    }
}
