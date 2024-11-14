use std::convert::Infallible;

use crate::{observer::Observer, patch::{Patch, PatchId}, turtle::{Turtle, TurtleId}};

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

pub enum Agent<'a> {
    Observer(&'a Observer),
    Turtle(&'a Turtle),
    Patch(&'a Patch),
    Link(Infallible /* TODO */),
}

pub enum AgentMut<'a> {
    Observer(&'a mut Observer),
    Turtle(&'a mut Turtle),
    Patch(&'a mut Patch),
    Link(Infallible /* TODO */),
}
