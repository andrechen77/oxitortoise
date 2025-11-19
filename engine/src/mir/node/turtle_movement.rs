//! Primitivees for moving turtles.

use derive_more::derive::Display;

use crate::mir::{Function, MirTy, NlAbstractTy, Node, NodeId, Nodes, Program};

#[derive(Debug, Display)]
#[display("TurtleRotate")]
pub struct TurtleRotate {
    /// The execution context to use.
    pub context: NodeId,
    /// The turtle to rotate.
    pub turtle: NodeId,
    /// The amount to rotate.
    pub angle: NodeId,
}

impl Node for TurtleRotate {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.turtle, self.angle]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Unit)
    }
}

#[derive(Debug, Display)]
#[display("TurtleForward")]
pub struct TurtleForward {
    /// The execution context to use.
    pub context: NodeId,
    /// The turtle to move.
    pub turtle: NodeId,
    /// The distance to move.
    pub distance: NodeId,
}

impl Node for TurtleForward {
    fn is_pure(&self) -> bool {
        false
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.turtle, self.distance]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Unit)
    }
}

#[derive(Debug, Display)]
#[display("CanMove")]
pub struct CanMove {
    /// The execution context to use.
    pub context: NodeId,
    /// The turtle to check movement for
    pub turtle: NodeId,
    /// The distance to check
    pub distance: NodeId,
}

impl Node for CanMove {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.turtle, self.distance]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Boolean)
    }

    // TODO(mvp_ants) add transformation to turn the node into "patch ahead != nobody"
}

#[derive(Debug, Display)]
pub enum PatchLocRelation {
    LeftAhead,
    RightAhead,
}

#[derive(Debug, Display)]
#[display("PatchNearby {relative_loc:?}")]
pub struct PatchRelative {
    /// The execution context to use.
    pub context: NodeId,
    /// The location to check relative to the patch
    pub relative_loc: PatchLocRelation,
    /// The turtle to check from
    pub turtle: NodeId,
    /// The distance to check
    pub distance: NodeId,
    /// The heading to check
    pub heading: NodeId,
}

impl Node for PatchRelative {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.context, self.turtle, self.distance, self.heading]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Patch)
    }
}
