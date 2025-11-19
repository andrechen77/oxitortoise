//! Commands and reporters that interact with the RNG.

use derive_more::derive::Display;

use crate::mir::{Function, MirTy, NlAbstractTy, Node, NodeId, Nodes, Program};

/// Returns a random integer between 0 (inclusive) and bound (exclusive)
#[derive(Debug, Display)]
#[display("RandomInt")]
pub struct RandomInt {
    /// The execution context to use.
    pub context: NodeId,
    pub bound: NodeId,
}

impl Node for RandomInt {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.bound]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Float)
    }
}
