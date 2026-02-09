//! A node representing a call to a user-defined function.

use std::fmt::{self, Write};

use pretty_print::PrettyPrinter;

use crate::{
    exec::jit::InstallLir,
    mir::{FunctionId, MirTy, Node, NodeId, Program, WriteLirError, build_lir::LirInsnBuilder},
};

#[derive(Debug)]
pub struct CallUserFn {
    /// The function being called.
    pub target: FunctionId,
    /// The arguments to the function.
    pub args: Vec<NodeId>,
}

impl Node for CallUserFn {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<(&'static str, NodeId)> {
        self.args.iter().map(|&id| ("arg", id)).collect()
    }

    fn output_type(&self, program: &Program, _fn_id: FunctionId) -> MirTy {
        program.functions[self.target].return_ty.clone()
    }

    fn write_lir_execution<I: InstallLir>(
        &self,
        program: &Program,
        my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let lir_fn_id = lir_builder.program_builder.available_user_functions[&self.target];
        let (_input_type, output_type) =
            &lir_builder.program_builder.function_signatures[&lir_fn_id];

        let mut args = Vec::new();
        for arg in &self.args {
            args.extend(lir_builder.get_node_results::<I>(program, *arg));
        }

        let insn = lir::InsnKind::CallUserFunction {
            function: lir_fn_id,
            output_type: output_type.clone(),
            args: args.into_boxed_slice(),
        };

        let pc = lir_builder.push_lir_insn(insn);
        let output_vals = (0..output_type.len()).map(|i| lir::ValRef(pc, i as u8)).collect();
        lir_builder.node_to_lir.insert(my_node_id, output_vals);
        Ok(())
    }

    fn pretty_print(&self, program: &Program, mut out: impl fmt::Write) -> fmt::Result {
        PrettyPrinter::new(&mut out).add_struct("CallUserFn", |p| {
            p.add_field("target", |p| write!(p, "{:?}", self.target))?;
            if let Some(fn_name) = program.functions[self.target].debug_name.as_deref() {
                p.add_comment(fn_name)?;
            }
            Ok(())
        })
    }
}
