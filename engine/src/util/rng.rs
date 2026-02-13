use std::{ops::Deref, sync::Mutex};

mod mersenne_twister;

pub trait Rng {
    /// Returns a random integer between 0 (inclusive) and `max` (exclusive).
    fn next_int(&mut self, max: i64) -> i64;
}

impl<T, R> Rng for T
where
    R: Rng + ?Sized,
    T: Deref<Target = Mutex<R>>,
{
    fn next_int(&mut self, max: i64) -> i64 {
        self.lock().unwrap().next_int(max)
    }
}

// The canonical RNG to use for this engine. This is aliased here to make it
// easier to change the RNG used in the future, and really only exists for
// debugging purposes. Feel free to remove.
pub type CanonRng = mersenne_twister::MersenneTwister;
