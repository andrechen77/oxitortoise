//! A node representing a call to a user-defined function.

use derive_more::derive::Display;

use crate::mir::{EffectfulNode, FunctionId, MirType, NodeId};

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
        program: &crate::mir::Program,
        _function: &crate::mir::Function,
        _nodes: &crate::mir::Nodes,
    ) -> MirType {
        program.functions[self.target].borrow().return_ty.clone()
    }
}
