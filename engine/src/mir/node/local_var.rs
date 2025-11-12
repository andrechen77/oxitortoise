use derive_more::derive::Display;
use lir::smallvec::smallvec;

use crate::mir::{
    EffectfulNode, Function, LocalId, MirType, NetlogoAbstractType, NodeId, Nodes, Program,
    WriteLirError,
    build_lir::{LirInsnBuilder, LocalLocation},
};

#[derive(Debug, Display)]
#[display("GetLocalVar {local_id:?}")]
pub struct GetLocalVar {
    /// The id of the local variable being gotten.
    pub local_id: LocalId,
}

impl EffectfulNode for GetLocalVar {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![]
    }

    fn output_type(&self, _program: &Program, function: &Function, _nodes: &Nodes) -> MirType {
        function.locals[self.local_id].ty.clone()
    }

    fn write_lir_execution(
        &self,
        my_node_id: NodeId,
        _nodes: &Nodes,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        match lir_builder.local_to_lir[&self.local_id] {
            LocalLocation::Stack { offset } => {
                let _ = offset;
                todo!("TODO(mvp) write LIR code to load the value from the stack")
            }
            LocalLocation::Var { var_id } => {
                // FIXME this should account for the case the MIR value spans
                // multiple LIR values
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
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.value]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirType {
        MirType::Abstract(NetlogoAbstractType::Unit)
    }
}
