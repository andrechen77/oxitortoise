use std::convert::Infallible;

use crate::{patch::PatchId, turtle::TurtleId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentId {
    Observer,
    Turtle(TurtleId),
    Patch(PatchId),
    Link(Infallible /* TODO */),
}

impl From<TurtleId> for AgentId {
    fn from(turtle: TurtleId) -> Self {
        AgentId::Turtle(turtle)
    }
}

impl From<PatchId> for AgentId {
    fn from(patch: PatchId) -> Self {
        AgentId::Patch(patch)
    }
}
