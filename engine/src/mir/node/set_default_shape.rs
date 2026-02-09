//! The `set-default-shape` command.

use std::fmt;

use pretty_print::PrettyPrinter;

use crate::{
    exec::jit::InstallLir,
    mir::{
        FunctionId, MirTy, NlAbstractTy, Node, NodeId, Program, WriteLirError,
        build_lir::LirInsnBuilder,
    },
};

#[derive(Debug)]
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

    fn dependencies(&self) -> Vec<(&'static str, NodeId)> {
        vec![("breed", self.breed), ("shape", self.shape)]
    }

    fn output_type(&self, _program: &Program, _fn_id: FunctionId) -> MirTy {
        NlAbstractTy::Unit.into()
    }

    fn write_lir_execution<I: InstallLir>(
        &self,
        _program: &Program,
        _my_node_id: NodeId,
        _lir_builder: &mut LirInsnBuilder,
    ) -> Result<(), WriteLirError> {
        // TODO(mvp) write LIR code to set the default shape for the breed
        Ok(())
    }

    fn pretty_print(&self, _program: &Program, mut out: impl fmt::Write) -> fmt::Result {
        PrettyPrinter::new(&mut out).add_struct("SetDefaultShape", |_| Ok(()))
    }
}
