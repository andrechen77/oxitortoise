use crate::turtle::TurtleId;

pub enum AgentId {
    Observer,
    Turtle(TurtleId),
    Patch(PatchId),
    Link(LinkId),
}

impl From<TurtleId> for AgentId {
    fn from(id: TurtleId) -> Self {
        AgentId::Turtle(id)
    }
}
impl From<PatchId> for AgentId {
    fn from(id: PatchId) -> Self {
        AgentId::Patch(id)
    }
}
impl From<LinkId> for AgentId {
    fn from(id: LinkId) -> Self {
        AgentId::Link(id)
    }
}

/// A reference to a patch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PatchId {
    // The index of the patch in the [`Patches`] struct.
    grid_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LinkId {
    from: TurtleId,
    to: TurtleId,
    // TODO distinguish by breed (or lack thereof)
}
