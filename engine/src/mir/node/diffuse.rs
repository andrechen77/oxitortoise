//! The `diffuse` command.

use derive_more::derive::Display;

use crate::{
    exec::jit::host_fn,
    mir::{
        Function, MirTy, NlAbstractTy, Node, NodeId, Nodes, Program, WriteLirError,
        build_lir::LirInsnBuilder,
    },
    sim::patch::PatchVarDesc,
};

#[derive(Debug, Display)]
#[display("Diffuse {variable:?}")]
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

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.amt]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Unit)
    }

    fn write_lir_execution(
        &self,
        program: &Program,
        function: &Function,
        nodes: &Nodes,
        _my_node_id: NodeId,
        lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        let &[ctx] = lir_builder.get_node_results(program, function, nodes, self.context) else {
            panic!("expected node outputting context pointer to be a single LIR value")
        };

        let (field_desc, _) = program
            .patch_schema
            .as_ref()
            .expect("patch schema should exist at this point")
            .field_desc_and_offset(self.variable);

        let field_desc = lir_builder.push_lir_insn(lir::InsnKind::Const(lir::Const {
            ty: lir::ValType::I16,
            value: field_desc.to_u16() as u64,
        }));

        let &[amt] = lir_builder.get_node_results(program, function, nodes, self.amt) else {
            panic!("expected node outputting amt to be a single LIR value")
        };

        lir_builder.push_lir_insn(lir::generate_host_function_call(
            host_fn::DIFFUSE_8_SINGLE_VARIABLE_BUFFER,
            Box::new([ctx, lir::ValRef(field_desc, 0), amt]),
        ));

        Ok(())
    }
}
