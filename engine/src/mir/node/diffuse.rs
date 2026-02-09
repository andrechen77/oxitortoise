//! The `diffuse` command.

use std::fmt::{self, Write};

use pretty_print::PrettyPrinter;

use crate::{
    exec::jit::InstallLir,
    mir::{
        FunctionId, MirTy, NlAbstractTy, Node, NodeId, Program, WriteLirError,
        build_lir::LirInsnBuilder,
    },
    sim::patch::PatchVarDesc,
};

#[derive(Debug)]
pub struct Diffuse {
    /// The execution context to use.
    pub context: NodeId,
    /// The patch variable to diffuse.
    pub variable: PatchVarDesc,
    /// The amount of the variable to diffuse.
    pub amt: NodeId,
}

impl Node for Diffuse {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<(&'static str, NodeId)> {
        vec![("ctx", self.context), ("amt", self.amt)]
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
        let &[ctx] = lir_builder.get_node_results::<I>(program, self.context) else {
            panic!("expected node outputting context pointer to be a single LIR value")
        };

        let (field_desc, _) = program
            .patch_schema
            .as_ref()
            .expect("patch schema should exist at this point")
            .field_desc_and_offset(self.variable);

        let field_desc = lir_builder
            .push_lir_insn(lir::InsnKind::Const(lir::Value::I32(field_desc.to_u16() as u32)));

        let &[amt] = lir_builder.get_node_results::<I>(program, self.amt) else {
            panic!("expected node outputting amt to be a single LIR value")
        };

        lir_builder.push_lir_insn(lir::generate_host_function_call(
            I::HOST_FUNCTION_TABLE.diffuse_8_single_variable_buffer,
            Box::new([ctx, lir::ValRef(field_desc, 0), amt]),
        ));

        Ok(())
    }

    fn pretty_print(&self, program: &Program, mut out: impl fmt::Write) -> fmt::Result {
        PrettyPrinter::new(&mut out).add_struct("Diffuse", |p| {
            p.add_field("variable", |p| write!(p, "{:?}", self.variable))?;
            if let PatchVarDesc::Custom(field) = self.variable {
                p.add_comment(&program.custom_patch_vars[field].name)?;
            }
            Ok(())
        })
    }
}
