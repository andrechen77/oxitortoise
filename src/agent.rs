use std::convert::Infallible;

use derive_more::derive::{From, TryInto};

use crate::{
    observer::Observer,
    patch::{Patch, PatchId},
    turtle::{Turtle, TurtleId},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, TryInto)]
pub enum AgentId {
    Observer,
    Turtle(TurtleId),
    Patch(PatchId),
    Link(Infallible /* TODO */),
}

#[derive(Debug, Clone, Copy, From, TryInto)]
pub enum Agent<'a> {
    Observer(&'a Observer),
    Turtle(&'a Turtle),
    Patch(&'a Patch),
    Link(Infallible /* TODO */),
}

#[derive(Debug, From, TryInto)]
pub enum AgentMut<'a> {
    Observer(&'a mut Observer),
    Turtle(&'a mut Turtle),
    Patch(&'a mut Patch),
    Link(Infallible /* TODO */),
}
