use std::{cell::RefCell, convert::Infallible, rc::Rc};

use derive_more::derive::{From, TryInto};

use crate::sim::{
    observer::Observer,
    patch::{Patch, PatchId},
    turtle::{Turtle, TurtleId},
};

use super::{topology::Point, world::World};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, TryInto)]
pub enum AgentId {
    Observer,
    Turtle(TurtleId),
    Patch(PatchId),
    Link(Infallible /* TODO */),
}

#[derive(Debug, Clone, From, TryInto)]
pub enum Agent<'a> {
    Observer(&'a RefCell<Observer>),
    Turtle(Rc<RefCell<Turtle>>),
    Patch(&'a RefCell<Patch>),
    Link(Infallible /* TODO */),
}

pub trait AgentIndexIntoWorld {
    type Output<'w>;

    fn index_into_world<'w>(self, world: &'w World) -> Option<Self::Output<'w>>;
}

// A trait for getting the position of an agent in the world.
pub trait AgentPosition {
    fn position(&self) -> Point;
}
