//! Nodes for commands that perform operations on colors.

use derive_more::derive::Display;
use lir::smallvec::smallvec;

use crate::{
    exec::jit::host_fn,
    mir::{
        Function, MirTy, NlAbstractTy, Node, NodeId, Nodes, Program, WriteLirError,
        build_lir::LirInsnBuilder,
    },
};

/// https://docs.netlogo.org/dict/scale-color.html
#[derive(Debug, Display)]
#[display("ScaleColor")]
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

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.color, self.number, self.range1, self.range2]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Color)
    }

    fn write_lir_execution(
        &self,
        program: &Program,
        function: &Function,
        nodes: &Nodes,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[color] = lir_builder.get_node_results(program, function, nodes, self.color) else {
            panic!("expected node outputting color to be a single LIR value")
        };
        let &[number] = lir_builder.get_node_results(program, function, nodes, self.number) else {
            panic!("expected node outputting number to be a single LIR value")
        };
        let &[range1] = lir_builder.get_node_results(program, function, nodes, self.range1) else {
            panic!("expected node outputting range1 to be a single LIR value")
        };
        let &[range2] = lir_builder.get_node_results(program, function, nodes, self.range2) else {
            panic!("expected node outputting range2 to be a single LIR value")
        };
        let insn = lir::generate_host_function_call(
            host_fn::SCALE_COLOR,
            Box::new([color, number, range1, range2]),
        );
        let pc = lir_builder.push_lir_insn(insn);
        lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(pc, 0)]);
        Ok(())
    }
}
