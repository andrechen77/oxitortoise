use std::{cell::RefCell, fmt::Debug, rc::Rc};

use rand::{rngs::StdRng, Rng as _, SeedableRng as _};

// the implementation of this is basic for prototyping purposes, but it should
// be made to match the behavior of NetLogo Desktop

pub trait NextInt: Debug {
    /// Returns a random integer between 0 (inclusive) and `max` (exclusive).
    fn next_int(&mut self, max: i64) -> i64;
}

impl<N: NextInt + ?Sized> NextInt for Rc<RefCell<N>> {
    fn next_int(&mut self, max: i64) -> i64 {
        self.borrow_mut().next_int(max)
    }
}

#[derive(Debug)]
pub struct RandIntGenerator {
    // TODO
    rng: StdRng,
}

impl RandIntGenerator {
    pub fn new() -> Self {
        Self {
            rng: StdRng::from_entropy(),
        }
    }
}

impl NextInt for RandIntGenerator {
    fn next_int(&mut self, max: i64) -> i64 {
        self.rng.gen_range(0..max)
    }
}

impl Default for RandIntGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// The canonical RNG to use for this engine. This is aliased here to make it
// easier to change the RNG used in the future, and really only exists for
// debugging purposes. Feel free to remove.
pub type CanonRng = RandIntGenerator;
