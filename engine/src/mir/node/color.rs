//! Nodes for commands that perform operations on colors.

use derive_more::derive::Display;

use crate::mir::{EffectfulNode, Function, MirTy, NlAbstractTy, NodeId, Nodes, Program};

/// https://docs.netlogo.org/dict/scale-color.html
#[derive(Debug, Display)]
#[display("ScaleColor")]
pub struct ScaleColor {
    pub color: NodeId,
    pub number: NodeId,
    pub range1: NodeId,
    pub range2: NodeId,
}

impl EffectfulNode for ScaleColor {
    fn is_pure(&self) -> bool {
        true
    }

    fn dependencies(&self) -> Vec<NodeId> {
        vec![self.color, self.number, self.range1, self.range2]
    }

    fn output_type(&self, _program: &Program, _function: &Function, _nodes: &Nodes) -> MirTy {
        MirTy::Abstract(NlAbstractTy::Color)
    }
}
