//! Eagerly evaluated agentsets.

use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::{
    sim::{agent::AgentId, turtle::TurtleId},
    util::{
        rng::CanonRng,
        shuffle_iterator::{ShuffledMut, ShuffledOwned},
    },
};

/// # Note
///
/// The "set" in `AgentSet` is intentionally not capitalized, as the NetLogo
/// language refers to the object as a single word: "agentset".
#[derive(Debug)]
pub enum Agentset {
    Turtles(TurtleSet),
}

pub trait IterateAgentset {
    type Item: Into<AgentId>;

    fn iter(&mut self, rng: Rc<RefCell<CanonRng>>) -> impl Iterator<Item = Self::Item>;

    fn into_iter(self, rng: Rc<RefCell<CanonRng>>) -> impl Iterator<Item = Self::Item>;
}

#[derive(Debug, Default, Clone)]
pub struct TurtleSet {
    turtles: Vec<TurtleId>,
}

impl TurtleSet {
    pub fn new(turtles: Vec<TurtleId>) -> Self {
        Self { turtles }
    }
}

impl IterateAgentset for TurtleSet {
    type Item = TurtleId;

    fn iter(&mut self, rng: Rc<RefCell<CanonRng>>) -> impl Iterator<Item = Self::Item> {
        ShuffledMut::new(&mut self.turtles, rng).map(|id| *id)
    }

    fn into_iter(self, rng: Rc<RefCell<CanonRng>>) -> impl Iterator<Item = Self::Item> {
        ShuffledOwned::new(VecDeque::from(self.turtles), rng)
    }
}
