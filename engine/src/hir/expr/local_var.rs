//! Nodes for getting and setting local variables.

use std::fmt;

use pretty_print::PrettyPrinter;

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, LocalId, NlAbstractTy, Program},
    mir,
};

#[derive(Debug, Clone)]
pub struct GetLocalVar {
    pub local_id: LocalId,
}

impl Expr for GetLocalVar {
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
        todo!("TODO(mvp) GetLocalVar output type inference")
    }

    fn visit_children(&self, _visitor: impl FnMut(&ExprKind)) {
        // no children
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for GetLocalVar")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        _program: &Program,
    ) -> fmt::Result {
        p.add_fn_call("get_local", |p| {
            p.add_fn_arg(self.local_id.0)?;
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
    fn output_type(&self, _program: &Program) -> NlAbstractTy {
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
        program: &Program,
    ) -> fmt::Result {
        let SetLocalVar { local_id, value } = self;
        p.add_fn_call("set_local", |p| {
            p.add_fn_arg(local_id.0)?;
            p.add_fn_arg_with(|p| value.pretty_print(p, program))?;
            Ok(())
        })
    }
}
