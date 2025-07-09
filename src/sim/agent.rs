use std::convert::Infallible;

use derive_more::derive::{From, TryInto};

use crate::{
    sim::{
        observer::Observer,
        patch::{Patch, PatchId},
    },
    util::cell::RefCell,
};

use super::{
    topology::Point,
    turtle::{Turtle, TurtleId},
    world::World,
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
    Observer(&'a RefCell<Observer>),
    Turtle(&'a Turtle),
    Patch(&'a Patch),
    Link(Infallible /* TODO */),
}
