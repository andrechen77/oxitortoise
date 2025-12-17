//! Primitives for interacting with the tick counter.

use derive_more::derive::Display;
use lir::smallvec::smallvec;

use crate::{
    exec::jit::host_fn,
    mir::{
        FunctionId, MirTy, NlAbstractTy, Node, NodeId, Program, WriteLirError,
        build_lir::LirInsnBuilder,
    },
};

#[derive(Debug, Display)]
#[display("ResetTicks {context:?}")]
pub struct ResetTicks {
    /// The execution context to use.
    pub context: NodeId,
}

impl Node for ResetTicks {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        NlAbstractTy::Unit.into()
    }

    fn write_lir_execution(
        &self,
        program: &Program,
        _my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[ctx_ptr] = lir_builder.get_node_results(program, self.context) else {
            panic!("expected node outputting context pointer to be a single LIR value")
        };
        lir_builder.push_lir_insn(lir::generate_host_function_call(
            host_fn::RESET_TICKS,
            Box::new([ctx_ptr]),
        ));
        Ok(())
    }
}

#[derive(Debug, Display)]
#[display("AdvanceTick")]
pub struct AdvanceTick {
    /// The context whose tick is being advanced.
    pub context: NodeId,
}

impl Node for AdvanceTick {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        NlAbstractTy::Unit.into()
    }

    fn write_lir_execution(
        &self,
        program: &Program,
        _my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[ctx_ptr] = lir_builder.get_node_results(program, self.context) else {
            panic!("expected node outputting context pointer to be a single LIR value")
        };
        lir_builder.push_lir_insn(lir::generate_host_function_call(
            host_fn::ADVANCE_TICK,
            Box::new([ctx_ptr]),
        ));
        Ok(())
    }
}

#[derive(Debug, Display)]
#[display("GetTick")]
pub struct GetTick {
    /// The context whose tick is being gotten.
    pub context: NodeId,
}

impl Node for GetTick {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        NlAbstractTy::Float.into()
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
        let pc = lir_builder.push_lir_insn(lir::generate_host_function_call(
            host_fn::GET_TICK,
            Box::new([ctx_ptr]),
        ));

        lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(pc, 0)]);

        Ok(())
    }
}
