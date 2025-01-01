use std::{fmt::Debug, rc::Rc};

use crate::util::cell::RefCell;

mod mersenne_twister;

pub trait Rng: Debug {
    /// Returns a random integer between 0 (inclusive) and `max` (exclusive).
    fn next_int(&mut self, max: i64) -> i64;
}

impl<R: Rng + ?Sized> Rng for Rc<RefCell<R>> {
    fn next_int(&mut self, max: i64) -> i64 {
        self.borrow_mut().next_int(max)
    }
}

// The canonical RNG to use for this engine. This is aliased here to make it
// easier to change the RNG used in the future, and really only exists for
// debugging purposes. Feel free to remove.
pub type CanonRng = mersenne_twister::MersenneTwister;
