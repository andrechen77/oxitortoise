//! Commands and reporters that interact with the RNG.

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

/// Returns a random integer between 0 (inclusive) and bound (exclusive)
#[derive(Debug)]
pub struct RandomInt {
    /// The execution context to use.
    pub context: NodeId,
    pub bound: NodeId,
}

impl Node for RandomInt {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<(&'static str, NodeId)> {
        vec![("ctx", self.context), ("bound", self.bound)]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        NlAbstractTy::Float.into()
    }

    fn write_lir_execution<I: InstallLir>(
        &self,
        program: &Program,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[ctx_ptr] = lir_builder.get_node_results::<I>(program, self.context) else {
            panic!("expected node outputting context pointer to be a single LIR value")
        };
        let &[bound] = lir_builder.get_node_results::<I>(program, self.bound) else {
            panic!("expected node outputting bound to be a single LIR value")
        };
        let pc = lir_builder.push_lir_insn(lir::generate_host_function_call(
            I::HOST_FUNCTION_TABLE.random_int,
            Box::new([ctx_ptr, bound]),
        ));
        lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(pc, 0)]);
        Ok(())
    }

    fn pretty_print(&self, _program: &Program, mut out: impl fmt::Write) -> fmt::Result {
        PrettyPrinter::new(&mut out).add_struct("RandomInt", |_| Ok(()))
    }
}
