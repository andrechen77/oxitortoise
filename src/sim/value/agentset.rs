//! Eagerly evaluated agentsets.

use std::collections::VecDeque;

use crate::{
    sim::{agent::AgentIndexIntoWorld, patch::PatchId, turtle::TurtleId, world::World},
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
    type Item: AgentIndexIntoWorld;

    fn iter(&mut self, world: &World, rng: &RefCell<CanonRng>) -> impl Iterator<Item = Self::Item>;

    fn into_iter(self, world: &World, rng: &RefCell<CanonRng>) -> impl Iterator<Item = Self::Item>;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AllPatches;

impl IterateAgentset for AllPatches {
    type Item = PatchId;

    fn iter(&mut self, world: &World, rng: &RefCell<CanonRng>) -> impl Iterator<Item = Self::Item> {
        Self.into_iter(world, rng)
    }

    fn into_iter(self, world: &World, rng: &RefCell<CanonRng>) -> impl Iterator<Item = Self::Item> {
        let patch_ids: VecDeque<PatchId> = world.patches.patch_ids_iter().collect();
        ShuffledOwned::new(patch_ids, rng)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AllTurtles;

impl IterateAgentset for AllTurtles {
    type Item = TurtleId;

    fn iter(&mut self, world: &World, rng: &RefCell<CanonRng>) -> impl Iterator<Item = Self::Item> {
        Self.into_iter(world, rng)
    }

    fn into_iter(self, world: &World, rng: &RefCell<CanonRng>) -> impl Iterator<Item = Self::Item> {
        let turtle_ids: VecDeque<TurtleId> = world.turtles.turtle_ids().into();
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
    type Item = TurtleId;

    fn iter<'s>(
        &'s mut self,
        _: &World,
        rng: &RefCell<CanonRng>,
    ) -> impl Iterator<Item = Self::Item> {
        ShuffledMut::new(&mut self.turtles, rng).map(|id| *id)
    }

    fn into_iter(self, _: &World, rng: &RefCell<CanonRng>) -> impl Iterator<Item = Self::Item> {
        ShuffledOwned::new(VecDeque::from(self.turtles), rng)
    }
}
