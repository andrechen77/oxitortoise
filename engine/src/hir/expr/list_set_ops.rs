//! Nodes for primitives that operate on lists and agentsets.

use std::fmt;

use pretty_print::PrettyPrinter;

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, NlAbstractTy, Program},
    mir,
};

#[derive(Debug, Clone)]
pub struct OneOf {
    pub rng: Box<ExprKind>,
    pub operand: Box<ExprKind>,
}

impl Expr for OneOf {
    fn output_type(&self, program: &Program) -> NlAbstractTy {
        let operand_ty = self.operand.output_type(program);
        match operand_ty {
            NlAbstractTy::Agentset { agent_type } => *agent_type,
            NlAbstractTy::List { element_ty } => *element_ty,
            x => todo!("TODO(mvp) OneOf unsupported operand type: {:?}", x),
        }
    }

    fn visit_children(&self, mut visitor: impl FnMut(&ExprKind)) {
        visitor(&self.rng);
        visitor(&self.operand);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for OneOf")
    }

    fn pretty_print<W: fmt::Write>(
        &self,
        p: &mut PrettyPrinter<W>,
        program: &Program,
    ) -> fmt::Result {
        let OneOf { rng, operand } = self;
        p.add_fn_call("one_of", |p| {
            p.add_fn_arg_with(|p| rng.pretty_print(p, program))?;
            p.add_fn_arg_with(|p| operand.pretty_print(p, program))?;
            Ok(())
        })
    }
}
