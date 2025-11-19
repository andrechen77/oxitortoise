//! The `diffuse` command.

use derive_more::derive::Display;

use crate::{
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
        _program: &Program,
        _function: &Function,
        _nodes: &Nodes,
        _my_node_id: NodeId,
        _lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        todo!("TODO(mvp_ants) write LIR code to call a host function")
    }
}
