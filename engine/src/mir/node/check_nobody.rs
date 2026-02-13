use std::fmt::{self, Write};

use lir::smallvec::smallvec;
use pretty_print::PrettyPrinter;

use crate::{
    exec::jit::InstallLir,
    mir::{
        FunctionId, MirTy, NlAbstractTy, Node, NodeId, Program, WriteLirError,
        build_lir::LirInsnBuilder,
    },
    sim::patch::OptionPatchId,
    util::reflection::Reflect,
};

#[derive(Debug)]
pub struct CheckNobody {
    pub context: NodeId,
    pub agent: NodeId,
    pub negate: bool,
}

impl Node for CheckNobody {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<(&'static str, NodeId)> {
        vec![("ctx", self.context), ("agent", self.agent)]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        NlAbstractTy::Boolean.into()
    }

    fn write_lir_execution<I: InstallLir>(
        &self,
        program: &Program,
        my_node_id: NodeId,

        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let operand_type = program.nodes[self.agent].output_type(program, lir_builder.fn_id).repr();
        if operand_type == OptionPatchId::CONCRETE_TY {
            let &[agent] = lir_builder.get_node_results::<I>(program, self.agent) else {
                panic!("expected a node that outputs a patch ID to be a single LIR register");
            };

            let sentinel = lir_builder
                .push_lir_insn(lir::InsnKind::Const(lir::Value::I32(OptionPatchId::NOBODY.0)));
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

    fn pretty_print(&self, _program: &Program, mut out: impl fmt::Write) -> fmt::Result {
        PrettyPrinter::new(&mut out).add_struct("CheckNobody", |p| {
            p.add_field_with("negate", |p| write!(p, "{}", self.negate))
        })
    }
}
