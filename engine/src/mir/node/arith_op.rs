//! Nodes to represent basic arithmetic operations that should not be host
//! function calls.

use derive_more::derive::Display;
use lir::smallvec::smallvec;

use crate::mir::{
    Function, MirTy, NlAbstractTy, Node, NodeId, Nodes, Program, WriteLirError,
    build_lir::LirInsnBuilder,
};

#[derive(Debug, Display)]
#[display("{_0:?}")]
pub enum BinaryOpcode {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
    And,
    Or,
}

#[derive(Debug, Display)]
#[display("{op:?}")]
pub struct BinaryOperation {
    pub op: BinaryOpcode,
    pub lhs: NodeId,
    pub rhs: NodeId,
}

impl Node for BinaryOperation {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.lhs, self.rhs]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(match self.op {
            BinaryOpcode::Add => NlAbstractTy::Numeric,
            BinaryOpcode::Sub => NlAbstractTy::Numeric,
            BinaryOpcode::Mul => NlAbstractTy::Numeric,
            BinaryOpcode::Div => NlAbstractTy::Numeric,
            BinaryOpcode::Lt => NlAbstractTy::Boolean,
            BinaryOpcode::Lte => NlAbstractTy::Boolean,
            BinaryOpcode::Gt => NlAbstractTy::Boolean,
            BinaryOpcode::Gte => NlAbstractTy::Boolean,
            BinaryOpcode::Eq => NlAbstractTy::Boolean,
            BinaryOpcode::And => NlAbstractTy::Boolean,
            BinaryOpcode::Or => NlAbstractTy::Boolean,
        })
    }

    fn write_lir_execution(
        &self,
        program: &Program,
        function: &Function,
        nodes: &Nodes,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        // TODO(mvp) be prepared for other possible input types and adjust
        // the implementation accordingly
        // TODO(mvp) assert that the types of the operands are compatible with the operation
        let _lhs_type = nodes[self.lhs].output_type(program, function, nodes).repr();
        let _rhs_type = nodes[self.rhs].output_type(program, function, nodes).repr();

        let &[lhs] = lir_builder.get_node_results(program, function, nodes, self.lhs) else {
            unimplemented!();
        };
        let &[rhs] = lir_builder.get_node_results(program, function, nodes, self.rhs) else {
            unimplemented!();
        };
        use BinaryOpcode as Op;
        let op = match self.op {
            Op::Add => lir::BinaryOpcode::FAdd,
            Op::Sub => lir::BinaryOpcode::FSub,
            Op::Mul => lir::BinaryOpcode::FMul,
            Op::Div => lir::BinaryOpcode::FDiv,
            Op::Lt => lir::BinaryOpcode::FLt,
            Op::Lte => lir::BinaryOpcode::FLte,
            Op::Gt => lir::BinaryOpcode::FGt,
            Op::Gte => lir::BinaryOpcode::FGte,
            Op::Eq => lir::BinaryOpcode::FEq,
            Op::And => lir::BinaryOpcode::And,
            Op::Or => lir::BinaryOpcode::Or,
        };

        let result = lir_builder.push_lir_insn(lir::InsnKind::BinaryOp { op, lhs, rhs });
        lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(result, 0)]);
        Ok(())
    }
}

#[derive(Debug, Display)]
#[display("{op:?}")]
pub enum UnaryOpcode {
    Neg,
    Not,
}

#[derive(Debug, Display)]
#[display("{op:?}")]
pub struct UnaryOp {
    pub op: UnaryOpcode,
    pub operand: NodeId,
}

impl Node for UnaryOp {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.operand]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(match self.op {
            UnaryOpcode::Neg => NlAbstractTy::Numeric,
            UnaryOpcode::Not => NlAbstractTy::Boolean,
        })
    }

    fn write_lir_execution(
        &self,
        program: &Program,
        function: &Function,
        nodes: &Nodes,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[operand] = lir_builder.get_node_results(program, function, nodes, self.operand)
        else {
            todo!("TODO(mvp) are there operands that are multi-register values?");
        };
        let op = match self.op {
            UnaryOpcode::Neg => lir::UnaryOpcode::FNeg,
            UnaryOpcode::Not => lir::UnaryOpcode::Not,
        };
        let result = lir_builder.push_lir_insn(lir::InsnKind::UnaryOp { op, operand });
        lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(result, 0)]);
        Ok(())
    }
}
