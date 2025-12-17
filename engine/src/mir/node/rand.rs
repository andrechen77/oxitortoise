//! Commands and reporters that interact with the RNG.

use derive_more::derive::Display;
use lir::smallvec::smallvec;

use crate::{
    exec::jit::host_fn,
    mir::{
        FunctionId, MirTy, NlAbstractTy, Node, NodeId, Program, WriteLirError,
        build_lir::LirInsnBuilder,
    },
};

/// Returns a random integer between 0 (inclusive) and bound (exclusive)
#[derive(Debug, Display)]
#[display("RandomInt")]
pub struct RandomInt {
    /// The execution context to use.
    pub context: NodeId,
    pub bound: NodeId,
}

impl Node for RandomInt {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.bound]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Float)
    }

    fn write_lir_execution(
        &self,
        program: &Program,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[ctx_ptr] = lir_builder.get_node_results(program, self.context) else {
            panic!("expected node outputting context pointer to be a single LIR value")
        };
        let &[bound] = lir_builder.get_node_results(program, self.bound) else {
            panic!("expected node outputting bound to be a single LIR value")
        };
        let pc = lir_builder.push_lir_insn(lir::generate_host_function_call(
            host_fn::RANDOM_INT,
            Box::new([ctx_ptr, bound]),
        ));
        lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(pc, 0)]);
        Ok(())
    }
}
