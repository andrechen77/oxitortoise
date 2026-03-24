//! Eagerly evaluated agentsets.

use std::collections::VecDeque;

use crate::{
    sim::{patch::PatchId, turtle::TurtleId, world::World},
    util::shuffle_iterator::ShuffledOwned,
};

pub type TurtleIterator = ShuffledOwned<TurtleId>;
pub type PatchIterator = ShuffledOwned<PatchId>;

pub fn shuffled_patches(world: &World) -> PatchIterator {
    let patch_ids: VecDeque<PatchId> = world.patches.patch_ids().collect();
    ShuffledOwned::new(patch_ids)
}

pub fn shuffled_turtles(world: &World) -> TurtleIterator {
    let turtle_ids: VecDeque<TurtleId> = world.turtles.turtle_ids().collect();
    ShuffledOwned::new(turtle_ids)
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
    pub fn into_shuffler(self) -> TurtleIterator {
        ShuffledOwned::new(VecDeque::from(self.turtles))
    }
}
