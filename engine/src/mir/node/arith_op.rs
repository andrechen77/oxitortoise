//! Nodes to represent basic arithmetic operations that should not be host
//! function calls.

use derive_more::derive::Display;

use crate::{
    mir::{
        Function, MirTy, NlAbstractTy, Node, NodeId, Nodes, Program, WriteLirError,
        build_lir::LirInsnBuilder,
    },
    sim::value::NlFloat,
    util::reflection::Reflect,
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
        _my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        // TODO(mvp) be prepared for other possible input types and adjust
        // the implementation accordingly
        assert_eq!(
            nodes[self.lhs].output_type(program, function, nodes).repr(),
            NlFloat::CONCRETE_TY
        );
        assert_eq!(
            nodes[self.rhs].output_type(program, function, nodes).repr(),
            NlFloat::CONCRETE_TY
        );

        let &[lhs] = lir_builder.get_node_results(program, function, nodes, self.lhs) else {
            unimplemented!();
        };
        let &[rhs] = lir_builder.get_node_results(program, function, nodes, self.rhs) else {
            unimplemented!();
        };
        let op = match self.op {
            BinaryOpcode::Add => lir::BinaryOpcode::FAdd,
            BinaryOpcode::Sub => lir::BinaryOpcode::FSub,
            BinaryOpcode::Mul => lir::BinaryOpcode::FMul,
            BinaryOpcode::Div => lir::BinaryOpcode::FDiv,
            BinaryOpcode::Lt => lir::BinaryOpcode::FLt,
            BinaryOpcode::Lte => lir::BinaryOpcode::FLte,
            BinaryOpcode::Gt => lir::BinaryOpcode::FGt,
            BinaryOpcode::Gte => lir::BinaryOpcode::FGte,
            BinaryOpcode::Eq => lir::BinaryOpcode::FEq,
            _ => todo!("TODO(mvp) implement other binary operations"),
        };
        lir_builder.push_lir_insn(lir::InsnKind::BinaryOp { op, lhs, rhs });
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
        _program: &Program,
        _function: &Function,
        _nodes: &Nodes,
        _my_node_id: NodeId,
        _lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        todo!("TODO(mvp_ants) write LIR code to perform the unary operation")
    }
}
