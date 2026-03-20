//! Nodes for primitives that operate on lists and agentsets.

use crate::{
    hir::{Expr, ExprKind, HirToMirFnBuilder, NlAbstractTy, Program},
    mir,
};

#[derive(Debug)]
pub struct OneOf {
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
        visitor(&self.operand);
    }

    fn write_mir_execution(&self, _builder: &mut HirToMirFnBuilder, _local_out: mir::LocalId) {
        todo!("TODO(mvp) write MIR execution for OneOf")
    }
}

