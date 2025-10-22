use derive_more::derive::Display;
use lir::smallvec::smallvec;
use slotmap::SlotMap;

use crate::{
    mir::{
        EffectfulNode, Function, MirType, NetlogoAbstractType, NodeId, Nodes,
        Program, build_lir::LirInsnBuilder,
    },
    sim::value::UnpackedDynBox,
};

#[derive(Debug, Display)]
#[display("Constant {value:?}")]
pub struct Constant {
    pub value: UnpackedDynBox,
}

impl EffectfulNode for Constant {
    fn has_side_effects(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![]
    }

    fn output_type(
        &self,
        _program: &Program,
        _function: &Function,
        _nodes: &Nodes,
    ) -> MirType {
        MirType::Abstract(match self.value {
            UnpackedDynBox::Int(_) => NetlogoAbstractType::Integer,
            UnpackedDynBox::Float(_) => NetlogoAbstractType::Float,
            _ => todo!(),
        })
    }

    fn write_lir_execution(
        &self,
        my_node_id: NodeId,
        _nodes: &SlotMap<NodeId, Box<dyn EffectfulNode>>,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), ()> {
        let _ = my_node_id;
        match self.value {
            UnpackedDynBox::Int(value) => {
                let pc = lir_builder.push_lir_insn(lir::InsnKind::Const(lir::Const {
                    value: value as u64,
                    ty: lir::ValType::I64,
                }));
                lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(pc, 0)]);
                Ok(())
            }
            UnpackedDynBox::Float(value) => {
                let pc = lir_builder.push_lir_insn(lir::InsnKind::Const(lir::Const {
                    value: f64::to_bits(value),
                    ty: lir::ValType::F64,
                }));
                lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(pc, 0)]);
                Ok(())
            }
            _ => todo!(),
        }
    }
}
