//! The `distancexy` operation.

use derive_more::derive::Display;

use crate::mir::{EffectfulNode, Function, MirTy, NlAbstractTy, NodeId, Nodes, Program};

#[derive(Debug, Display)]
#[display("Distancexy {x:?} {y:?}")]
pub struct Distancexy {
    /// The agent to get the distance from.
    pub agent: NodeId,
    /// The x coordinate.
    pub x: NodeId,
    /// The y coordinate.
    pub y: NodeId,
}

impl EffectfulNode for Distancexy {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.agent, self.x, self.y]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Float)
    }
}
