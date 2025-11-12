//! Eagerly evaluated agentsets.

use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::{
    sim::{patch::PatchId, turtle::TurtleId, world::World},
    util::{rng::CanonRng, shuffle_iterator::ShuffledOwned},
};

pub type TurtleIterator = ShuffledOwned<TurtleId, Rc<RefCell<CanonRng>>>;
pub type PatchIterator = ShuffledOwned<PatchId, Rc<RefCell<CanonRng>>>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AllPatches;

impl AllPatches {
    pub fn into_iter(self, world: &World, rng: Rc<RefCell<CanonRng>>) -> PatchIterator {
        let patch_ids: VecDeque<PatchId> = world.patches.patch_ids().collect();
        ShuffledOwned::new(patch_ids, rng)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AllTurtles;

impl AllTurtles {
    pub fn into_iter(self, world: &World, rng: Rc<RefCell<CanonRng>>) -> TurtleIterator {
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
    pub fn into_iter(self, rng: Rc<RefCell<CanonRng>>) -> TurtleIterator {
        ShuffledOwned::new(VecDeque::from(self.turtles), rng)
    }
}
