use std::fmt::Debug;

use rand::{rngs::StdRng, Rng as _, SeedableRng as _};

// the implementation of this is basic for prototyping purposes, but it should
// be made to match the behavior of NetLogo Desktop

pub trait NextInt: Debug {
    fn next_int(&mut self, max: i32) -> i32;
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
    fn next_int(&mut self, max: i32) -> i32 {
        self.rng.gen_range(0..max)
    }
}
