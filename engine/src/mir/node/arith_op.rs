use derive_more::derive::Display;

use crate::mir::{
    EffectfulNode, Function, MirType, NetlogoAbstractType, NodeId, Nodes, Program, WriteLirError,
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
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.lhs, self.rhs]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirType {
        let int_type = NetlogoAbstractType::Integer;
        let mir_int = MirType::Abstract(int_type.clone());

        let type_of = |node_id| _nodes[node_id].output_type(_program, _function, _nodes);

        let int_preserving_type = || {
            if type_of(self.lhs) == mir_int && type_of(self.rhs) == mir_int {
                int_type
            } else {
                NetlogoAbstractType::Numeric
            }
        };

        MirType::Abstract(match self.op {
            BinaryOpcode::Add => int_preserving_type(),
            BinaryOpcode::Sub => int_preserving_type(),
            BinaryOpcode::Mul => int_preserving_type(),
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
        _nodes: &Nodes,
        _lir_builder: &mut crate::mir::build_lir::LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        todo!("TODO(mvp_ants) write LIR code to perform the binary operation")
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
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.operand]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirType {
        MirType::Abstract(match self.op {
            UnaryOpcode::Neg => NetlogoAbstractType::Numeric,
            UnaryOpcode::Not => NetlogoAbstractType::Boolean,
        })
    }

    fn write_lir_execution(
        &self,
        _my_node_id: NodeId,
        _nodes: &Nodes,
        _lir_builder: &mut crate::mir::build_lir::LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        todo!("TODO(mvp_ants) write LIR code to perform the unary operation")
    }
}
