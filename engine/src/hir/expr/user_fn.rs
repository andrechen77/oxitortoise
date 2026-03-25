//! Nodes for a call to a user-defined function.

use std::fmt::{self, Write};

use pretty_print::PrettyPrinter;

use crate::{
    hir::{Expr, ExprKind, FunctionId, HirToMirFnBuilder, NlAbstractTy, Program},
    mir,
};

#[derive(Debug, Clone)]
pub struct CallUserFn {
    /// The function being called.
    pub target: FunctionId,
    /// The arguments to the function.
    pub args: Vec<Box<ExprKind>>,
}

impl Expr for CallUserFn {
    fn output_type(&self, program: &Program) -> NlAbstractTy {
        // The current HIR doesn't expose function signatures directly at this
        // expression layer, but the return type can be inferred from the
        // function body's output type.
        program.functions[&self.target].body.output_type(program)
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        for arg in &self.args {
            visitor(arg);
        }
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for CallUserFn")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        program: &Program,
    ) -> fmt::Result {
        let CallUserFn { target, args } = self;
        p.add_fn_call("call_user_fn", |p| {
            p.add_fn_arg_with(|p| {
                let label = program
                    .functions
                    .get(target)
                    .and_then(|f| f.debug_name.as_deref())
                    .unwrap_or("?");
                write!(p, "{:?} /* {} */", target, label)
            })?;
            for arg in args {
                p.add_fn_arg_with(|p| arg.pretty_print(p, program))?;
            }
            Ok(())
        })
    }
}
