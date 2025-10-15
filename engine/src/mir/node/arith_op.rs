use derive_more::derive::Display;
use slotmap::SlotMap;

use crate::mir::{
    EffectfulNode, Function, NetlogoAbstractAbstractType, NetlogoAbstractType, NodeId, Nodes,
    Program,
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

impl EffectfulNode for BinaryOperation {
    fn has_side_effects(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.lhs, self.rhs]
    }

    fn output_type(
        &self,
        _program: &Program,
        _function: &Function,
        _nodes: &Nodes,
    ) -> NetlogoAbstractAbstractType {
        // TODO int + int should be int
        NetlogoAbstractAbstractType::AbstractType(match self.op {
            BinaryOpcode::Add => NetlogoAbstractType::Numeric,
            BinaryOpcode::Sub => NetlogoAbstractType::Numeric,
            BinaryOpcode::Mul => NetlogoAbstractType::Numeric,
            BinaryOpcode::Div => NetlogoAbstractType::Numeric,
            BinaryOpcode::Lt => NetlogoAbstractType::Boolean,
            BinaryOpcode::Lte => NetlogoAbstractType::Boolean,
            BinaryOpcode::Gt => NetlogoAbstractType::Boolean,
            BinaryOpcode::Gte => NetlogoAbstractType::Boolean,
            BinaryOpcode::Eq => NetlogoAbstractType::Boolean,
            BinaryOpcode::And => NetlogoAbstractType::Boolean,
            BinaryOpcode::Or => NetlogoAbstractType::Boolean,
        })
    }

    fn write_lir_execution(
        &self,
        _my_node_id: NodeId,
        _nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _lir_builder: &mut crate::mir::build_lir::LirInsnBuilder,
    ) -> Result<(), ()> {
        todo!()
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

impl EffectfulNode for UnaryOp {
    fn has_side_effects(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.operand]
    }

    fn output_type(
        &self,
        _program: &Program,
        _function: &Function,
        _nodes: &Nodes,
    ) -> NetlogoAbstractAbstractType {
        NetlogoAbstractAbstractType::AbstractType(match self.op {
            UnaryOpcode::Neg => NetlogoAbstractType::Numeric,
            UnaryOpcode::Not => NetlogoAbstractType::Boolean,
        })
    }

    fn write_lir_execution(
        &self,
        _my_node_id: NodeId,
        _nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _lir_builder: &mut crate::mir::build_lir::LirInsnBuilder,
    ) -> Result<(), ()> {
        todo!()
    }
}
