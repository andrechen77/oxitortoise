//! A node representing a call to a user-defined function.

use derive_more::derive::Display;

use crate::mir::{EffectfulNode, FunctionId, NodeId};

#[derive(Debug, Display)]
#[display("CallUserFn {target:?}")]
pub struct CallUserFn {
    /// The function being called.
    pub target: FunctionId,
    /// The arguments to the function.
    pub args: Vec<NodeId>,
}

impl EffectfulNode for CallUserFn {
    fn has_side_effects(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        self.args.clone()
    }

    fn output_type(
        &self,
        _workspace: &crate::workspace::Workspace,
        _nodes: &slotmap::SlotMap<NodeId, Box<dyn EffectfulNode>>,
        _locals: &slotmap::SlotMap<crate::mir::LocalId, crate::mir::LocalDeclaration>,
    ) -> Option<crate::sim::value::NetlogoInternalType> {
        todo!("somehow look up the return type of the function")
    }
}
