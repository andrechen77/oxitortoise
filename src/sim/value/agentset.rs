//! Eagerly evaluated agentsets.

use std::{collections::VecDeque, rc::Rc};

use crate::{
    sim::{patch::PatchId, turtle::TurtleId, world::World},
    util::{
        cell::RefCell,
        rng::{CanonRng, Rng},
        shuffle_iterator::ShuffledOwned,
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

#[repr(transparent)]
pub struct AgentsetIterator<T, R> {
    iter: ShuffledOwned<T, R>,
}

impl<T, R> Iterator for AgentsetIterator<T, R>
where
    R: Rng,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub type TurtleIterator = AgentsetIterator<TurtleId, Rc<RefCell<CanonRng>>>;
pub type PatchIterator = AgentsetIterator<PatchId, Rc<RefCell<CanonRng>>>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AllPatches;

impl AllPatches {
    pub fn into_iter(self, world: &World, rng: Rc<RefCell<CanonRng>>) -> PatchIterator {
        let patch_ids: VecDeque<PatchId> = world.patches.patch_ids().collect();
        AgentsetIterator {
            iter: ShuffledOwned::new(patch_ids, rng),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AllTurtles;

impl AllTurtles {
    pub fn into_iter(self, world: &World, rng: Rc<RefCell<CanonRng>>) -> TurtleIterator {
        let turtle_ids: VecDeque<TurtleId> = world.turtles.turtle_ids().collect();
        AgentsetIterator {
            iter: ShuffledOwned::new(turtle_ids, rng),
        }
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
        AgentsetIterator {
            iter: ShuffledOwned::new(VecDeque::from(self.turtles), rng),
        }
    }
}
