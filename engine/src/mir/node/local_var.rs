use derive_more::derive::Display;
use lir::smallvec::{SmallVec, smallvec};
use slotmap::SlotMap;

use crate::mir::{
    EffectfulNode, LocalDeclaration, LocalId, NodeId,
    build_lir::{LirInsnBuilder, LocalLocation},
};

#[derive(Debug, Display)]
#[display("GetLocalVar {local_id:?}")]
pub struct GetLocalVar {
    /// The id of the local variable being gotten.
    pub local_id: LocalId,
}

impl EffectfulNode for GetLocalVar {
    fn has_side_effects(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![]
    }

    fn output_type(
        &self,
        _workspace: &crate::workspace::Workspace,
        _nodes: &slotmap::SlotMap<NodeId, Box<dyn EffectfulNode>>,
        locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<crate::sim::value::NetlogoInternalType> {
        Some(locals[self.local_id].ty.clone())
    }

    fn write_lir_execution(
        &self,
        my_node_id: NodeId,
        nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), ()> {
        match lir_builder.local_to_lir[&self.local_id] {
            LocalLocation::Stack { offset } => todo!(),
            LocalLocation::Var { var_id } => {
                // TODO this should be done in a loop in case the MIR value
                // spans multiple LIR values
                let pc = lir_builder.push_lir_insn(lir::InsnKind::VarLoad { var_id });
                lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(pc, 0)]);
                Ok(())
            }
        }
    }
}

#[derive(Debug, Display)]
#[display("SetLocalVar {local_id:?}")]
pub struct SetLocalVar {
    /// The id of the local variable being set.
    pub local_id: LocalId,
    /// The value to put into the local variable.
    pub value: NodeId,
}

impl EffectfulNode for SetLocalVar {
    fn has_side_effects(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.value]
    }

    fn output_type(
        &self,
        _workspace: &crate::workspace::Workspace,
        _nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &SlotMap<LocalId, LocalDeclaration>,
    ) -> Option<crate::sim::value::NetlogoInternalType> {
        None
    }
}
