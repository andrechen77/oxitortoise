//! Eagerly evaluated agentsets.

use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::{
    sim::{agent::AgentId, patch::PatchId, turtle::TurtleId, world::World},
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

    // TODO is it technically correct to use `+ 's` on the bound here?
    // should `use<'s>` be used instead?
    fn iter<'s>(
        &'s mut self,
        world: &World,
        rng: Rc<RefCell<CanonRng>>,
    ) -> impl Iterator<Item = Self::Item> + 's;

    fn into_iter(
        self,
        world: &World,
        rng: Rc<RefCell<CanonRng>>,
    ) -> impl Iterator<Item = Self::Item> + 'static;
}

pub struct AllPatches;

impl IterateAgentset for AllPatches {
    type Item = PatchId;

    fn iter<'s>(
        &'s mut self,
        world: &World,
        rng: Rc<RefCell<CanonRng>>,
    ) -> impl Iterator<Item = Self::Item> + 's {
        let patch_ids: VecDeque<PatchId> = world.patches.patch_ids_iter().collect();
        ShuffledOwned::new(patch_ids, rng)
    }

    fn into_iter(
        self,
        world: &World,
        rng: Rc<RefCell<CanonRng>>,
    ) -> impl Iterator<Item = Self::Item> + 'static {
        let patch_ids: VecDeque<PatchId> = world.patches.patch_ids_iter().collect();
        ShuffledOwned::new(patch_ids, rng)
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
        rng: Rc<RefCell<CanonRng>>,
    ) -> impl Iterator<Item = Self::Item> + 's {
        ShuffledMut::new(&mut self.turtles, rng).map(|id| *id)
    }

    fn into_iter(
        self,
        _: &World,
        rng: Rc<RefCell<CanonRng>>,
    ) -> impl Iterator<Item = Self::Item> + 'static {
        ShuffledOwned::new(VecDeque::from(self.turtles), rng)
    }
}