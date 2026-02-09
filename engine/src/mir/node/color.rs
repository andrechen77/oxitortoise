//! Nodes for commands that perform operations on colors.

use std::fmt;

use lir::smallvec::smallvec;
use pretty_print::PrettyPrinter;

use crate::{
    exec::jit::InstallLir,
    mir::{
        FunctionId, MirTy, NlAbstractTy, Node, NodeId, Program, WriteLirError,
        build_lir::LirInsnBuilder,
    },
};

/// https://docs.netlogo.org/dict/scale-color.html
#[derive(Debug)]
pub struct ScaleColor {
    pub color: NodeId,
    pub number: NodeId,
    pub range1: NodeId,
    pub range2: NodeId,
}

impl Node for ScaleColor {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<(&'static str, NodeId)> {
        vec![("color", self.color), ("num", self.number), ("r1", self.range1), ("r2", self.range2)]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        NlAbstractTy::Color.into()
    }

    fn write_lir_execution<I: InstallLir>(
        &self,
        program: &Program,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[color] = lir_builder.get_node_results::<I>(program, self.color) else {
            panic!("expected node outputting color to be a single LIR value")
        };
        let &[number] = lir_builder.get_node_results::<I>(program, self.number) else {
            panic!("expected node outputting number to be a single LIR value")
        };
        let &[range1] = lir_builder.get_node_results::<I>(program, self.range1) else {
            panic!("expected node outputting range1 to be a single LIR value")
        };
        let &[range2] = lir_builder.get_node_results::<I>(program, self.range2) else {
            panic!("expected node outputting range2 to be a single LIR value")
        };
        let insn = lir::generate_host_function_call(
            I::HOST_FUNCTION_TABLE.scale_color,
            Box::new([color, number, range1, range2]),
        );
        let pc = lir_builder.push_lir_insn(insn);
        lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(pc, 0)]);
        Ok(())
    }

    fn pretty_print(&self, _program: &Program, mut out: impl fmt::Write) -> fmt::Result {
        PrettyPrinter::new(&mut out).add_struct("ScaleColor", |_| Ok(()))
    }
}
