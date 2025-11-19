use derive_more::derive::Display;
use lir::smallvec::smallvec;

use crate::{
    mir::{
        EffectfulNode, Function, MirTy, NlAbstractTy, NodeId, Nodes, Program, WriteLirError,
        build_lir::LirInsnBuilder,
    },
    sim::value::UnpackedDynBox,
};

#[derive(Debug, Display)]
#[display("Constant {value:?}")]
pub struct Constant {
    pub value: UnpackedDynBox,
}

impl EffectfulNode for Constant {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(match self.value {
            UnpackedDynBox::Float(_) => NlAbstractTy::Float,
            _ => todo!("TODO(mvp) include all other variants"),
        })
    }

    fn write_lir_execution(
        &self,
        _program: &Program,
        _function: &Function,
        _nodes: &Nodes,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let _ = my_node_id;
        match self.value {
            UnpackedDynBox::Float(value) => {
                let pc = lir_builder.push_lir_insn(lir::InsnKind::Const(lir::Const {
                    value: f64::to_bits(value),
                    ty: lir::ValType::F64,
                }));
                lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(pc, 0)]);
                Ok(())
            }
            _ => todo!("TODO(mvp) include other variants"),
        }
    }
}

#[derive(Debug, Display)]
#[display("ListLiteral {items:?}")]
pub struct ListLiteral {
    pub items: Vec<NodeId>,
}

impl EffectfulNode for ListLiteral {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        self.items.clone()
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::List { element_ty: Box::new(NlAbstractTy::Top) })
    }
}
