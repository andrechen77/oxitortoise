//! Eagerly evaluated agentsets.

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use crate::{
    sim::{patch::PatchId, turtle::TurtleId, world::World},
    util::{rng::CanonRng, shuffle_iterator::ShuffledOwned},
};

pub type TurtleIterator = ShuffledOwned<TurtleId, Arc<Mutex<CanonRng>>>;
pub type PatchIterator = ShuffledOwned<PatchId, Arc<Mutex<CanonRng>>>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AllPatches;

impl AllPatches {
    pub fn into_iter(self, world: &World, rng: Arc<Mutex<CanonRng>>) -> PatchIterator {
        let patch_ids: VecDeque<PatchId> = world.patches.patch_ids().collect();
        ShuffledOwned::new(patch_ids, rng)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AllTurtles;

impl AllTurtles {
    pub fn into_iter(self, world: &World, rng: Arc<Mutex<CanonRng>>) -> TurtleIterator {
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

impl TurtleSet {
    pub fn into_iter(self, rng: Arc<Mutex<CanonRng>>) -> TurtleIterator {
        ShuffledOwned::new(VecDeque::from(self.turtles), rng)
    }
}
