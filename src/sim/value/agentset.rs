//! Eagerly evaluated agentsets.

use std::{collections::VecDeque, rc::Rc};

use crate::{
    sim::{patch::PatchId, turtle::TurtleId, world::World},
    util::{
        cell::RefCell,
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
    type AgentId;

    fn into_iter(
        self,
        world: &World,
        rng: Rc<RefCell<CanonRng>>,
    ) -> impl Iterator<Item = Self::AgentId> + 'static;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AllPatches;

impl IterateAgentset for AllPatches {
    type AgentId = PatchId;

    fn into_iter(
        self,
        world: &World,
        rng: Rc<RefCell<CanonRng>>,
    ) -> impl Iterator<Item = Self::AgentId> + 'static {
        let patch_ids: VecDeque<PatchId> = world.patches.patch_ids().collect();
        ShuffledOwned::new(patch_ids, rng)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AllTurtles;

impl IterateAgentset for AllTurtles {
    type AgentId = TurtleId;

    fn into_iter(
        self,
        world: &World,
        rng: Rc<RefCell<CanonRng>>,
    ) -> impl Iterator<Item = Self::AgentId> + 'static {
        let turtle_ids: VecDeque<TurtleId> = world.turtles.turtle_ids().collect();
        ShuffledOwned::new(turtle_ids, rng)
    }
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
    type AgentId = TurtleId;

    fn into_iter(
        self,
        _: &World,
        rng: Rc<RefCell<CanonRng>>,
    ) -> impl Iterator<Item = Self::AgentId> + 'static {
        ShuffledOwned::new(VecDeque::from(self.turtles), rng)
    }
}
