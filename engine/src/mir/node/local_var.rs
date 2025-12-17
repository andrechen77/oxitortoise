//! Nodes for getting and setting local variables.

use derive_more::derive::Display;
use lir::smallvec::smallvec;

use crate::mir::{
    FunctionId, LocalId, MirTy, NlAbstractTy, Node, NodeId, Program, WriteLirError,
    build_lir::{LirInsnBuilder, LocalLocation},
};

#[derive(Debug, Display)]
#[display("GetLocalVar {local_id:?}")]
pub struct GetLocalVar {
    /// The id of the local variable being gotten.
    pub local_id: LocalId,
}

impl Node for GetLocalVar {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![]
    }

    fn output_type(&self, program: &Program, _fn_id: FunctionId) -> MirTy {
        program.locals[self.local_id].ty.clone()
    }

    fn write_lir_execution(
        &self,
        _program: &Program,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        match lir_builder.local_to_lir[&self.local_id] {
            LocalLocation::Stack { offset } => {
                let _ = offset;
                todo!("TODO(mvp) write LIR code to load the value from the stack")
            }
            LocalLocation::Var { var_id } => {
                // FIXME this should account for the case the MIR value spans
                // multiple LIR values
                let pc = lir_builder.push_lir_insn(lir::InsnKind::VarLoad { var_id });
                lir_builder.node_to_lir.insert(my_node_id, smallvec![lir::ValRef(pc, 0)]);
                Ok(())
            }
        }
    }
}

#[derive(Debug, Display)]
#[display("SetLocalVar {local_id:?}")]
pub struct SetLocalVar {
    /// The id of the local variable being set.
    pub local_id: LocalId,
    /// The value to put into the local variable.
    pub value: NodeId,
}

impl Node for SetLocalVar {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.value]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Unit)
    }

    fn write_lir_execution(
        &self,
        program: &Program,
        _my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        // TODO(mvp) this should account for values that take up multiple LIR registers
        // let local_location = *lir_builder.local_to_lir.entry(self.local_id).or_insert_with(|| {
        //     // TODO(mvp) sometimes a variable should be stored on the stack
        //     let &[lir_type] = program.locals[self.local_id]
        //         .ty
        //         .repr()
        //         .info()
        //         .lir_repr
        //         .expect("local variable must have known ABI")
        //     else {
        //         todo!()
        //     };
        //     let var_id = lir_builder.product.local_vars.push_and_get_key(lir_type);
        //     LocalLocation::Var { var_id }
        // });
        let local_location = lir_builder.local_to_lir[&self.local_id];

        let &[value] = lir_builder.get_node_results(program, self.value) else { todo!() };

        match local_location {
            LocalLocation::Stack { offset: _ } => {
                todo!("TODO(mvp) write LIR code to load the value from the stack")
            }
            LocalLocation::Var { var_id } => {
                // FIXME this should account for the case the MIR value spans multiple LIR values
                lir_builder.push_lir_insn(lir::InsnKind::VarStore { var_id, value });
                Ok(())
            }
        }
    }
}
