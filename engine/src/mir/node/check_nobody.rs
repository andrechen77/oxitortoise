use derive_more::derive::Display;
use lir::smallvec::smallvec;

use crate::{
    mir::{
        FunctionId, MirTy, NlAbstractTy, Node, NodeId, Program, WriteLirError,
        build_lir::LirInsnBuilder,
    },
    sim::patch::OptionPatchId,
    util::reflection::Reflect,
};

#[derive(Debug, Display)]
#[display("CheckNobody negate={negate:?}")]
pub struct CheckNobody {
    pub context: NodeId,
    pub agent: NodeId,
    pub negate: bool,
}

impl Node for CheckNobody {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.agent]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        NlAbstractTy::Boolean.into()
    }

    fn write_lir_execution(
        &self,
        program: &Program,
        my_node_id: NodeId,

        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let operand_type =
            program.nodes[self.agent].output_type(program, lir_builder.fn_id).concrete.unwrap();
        if operand_type == OptionPatchId::CONCRETE_TY {
            let &[agent] = lir_builder.get_node_results(program, self.agent) else {
                panic!("expected a node that outputs a patch ID to be a single LIR register");
            };

            let sentinel = lir_builder.push_lir_insn(lir::InsnKind::Const(lir::Const {
                ty: lir::ValType::I32,
                value: OptionPatchId::NOBODY.0 as u64,
            }));

            let opcode = if self.negate { lir::BinaryOpcode::INeq } else { lir::BinaryOpcode::IEq };
            let condition = lir_builder.push_lir_insn(lir::InsnKind::BinaryOp {
                op: opcode,
                lhs: agent,
                rhs: lir::ValRef(sentinel, 0),
            });

            lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(condition, 0)]);

            Ok(())
        } else {
            todo!("TODO(mvp) write this for other types")
        }
    }
}
