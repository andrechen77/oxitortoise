use derive_more::derive::Display;
use slotmap::SlotMap;

use crate::mir::{EffectfulNode, LocalDeclaration, LocalId, NodeId};

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
}

#[derive(Debug, Display)]
#[display("{op:?}")]
pub struct BinaryOperation {
    op: BinaryOpcode,
    lhs: NodeId,
    rhs: NodeId,
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
        _workspace: &crate::workspace::Workspace,
        _nodes: &slotmap::SlotMap<crate::mir::NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<crate::sim::value::NetlogoInternalType> {
        todo!("match on internal variant and operand types")
    }

    fn write_lir_execution(
        &self,
        my_node_id: NodeId,
        nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        lir_builder: &mut crate::mir::build_lir::LirInsnBuilder,
    ) -> Result<(), ()> {
        todo!()
    }
}
