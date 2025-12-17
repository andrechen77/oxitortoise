//! The `set-default-shape` command.

use derive_more::derive::Display;

use crate::mir::{
    FunctionId, MirTy, NlAbstractTy, Node, NodeId, Program, WriteLirError,
    build_lir::LirInsnBuilder,
};

#[derive(Debug, Display)]
#[display("SetDefaultShape")]
pub struct SetDefaultShape {
    /// The breed to set the default shape for.
    pub breed: NodeId,
    /// The shape to set.
    pub shape: NodeId,
}

impl Node for SetDefaultShape {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.breed, self.shape]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        NlAbstractTy::Unit.into()
    }

    fn write_lir_execution(
        &self,
        _program: &Program,
        _my_node_id: NodeId,
        _lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        // TODO(mvp) write LIR code to set the default shape for the breed
        Ok(())
    }
}
