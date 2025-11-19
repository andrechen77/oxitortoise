//! A node representing a call to a user-defined function.

use derive_more::derive::Display;

use crate::mir::{EffectfulNode, FunctionId, MirTy, NodeId};

#[derive(Debug, Display)]
#[display("CallUserFn {target:?}")]
pub struct CallUserFn {
    /// The function being called.
    pub target: FunctionId,
    /// The arguments to the function.
    pub args: Vec<NodeId>,
}

impl EffectfulNode for CallUserFn {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        self.args.clone()
    }

    fn output_type(
        &self,
        program: &crate::mir::Program,
        _function: &crate::mir::Function,
        _nodes: &crate::mir::Nodes,
    ) -> MirTy {
        program.functions[self.target].borrow().return_ty.clone()
    }

    fn write_lir_execution(
        &self,
        program: &crate::mir::Program,
        function: &crate::mir::Function,
        nodes: &crate::mir::Nodes,
        _my_node_id: NodeId,
        lir_builder: &mut crate::mir::build_lir::LirInsnBuilder,
    ) -> Result<(), crate::mir::WriteLirError> {
        let lir_fn_id = lir_builder.program_builder.available_user_functions[&self.target];
        let (_input_type, output_type) =
            &lir_builder.program_builder.function_signatures[&lir_fn_id];

        let mut args = Vec::new();
        for arg in &self.args {
            args.extend(lir_builder.get_node_results(program, function, nodes, *arg));
        }

        let insn = lir::InsnKind::CallUserFunction {
            function: lir_fn_id,
            output_type: output_type.clone(),
            args: args.into_boxed_slice(),
        };

        lir_builder.push_lir_insn(insn);
        Ok(())
    }
}
