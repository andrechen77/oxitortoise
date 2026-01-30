//! The `clear-all` command and friends.

use derive_more::derive::Display;

use crate::{
    exec::jit::InstallLir,
    mir::{
        FunctionId, MirTy, NlAbstractTy, Node, NodeId, Program, WriteLirError,
        build_lir::LirInsnBuilder,
    },
};

#[derive(Debug, Display)]
#[display("ClearAll")]
pub struct ClearAll {
    /// The execution context to use.
    pub context: NodeId,
}

impl Node for ClearAll {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        NlAbstractTy::Unit.into()
    }

    fn write_lir_execution<I: InstallLir>(
        &self,
        program: &Program,
        _my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[ctx_ptr] = lir_builder.get_node_results::<I>(program, self.context) else {
            panic!("expected node outputting context pointer to be a single LIR value")
        };
        lir_builder.push_lir_insn(lir::generate_host_function_call(
            I::HOST_FUNCTION_TABLE.clear_all,
            Box::new([ctx_ptr]),
        ));
        Ok(())
    }
}
