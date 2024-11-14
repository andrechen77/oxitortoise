use std::{cell::RefCell, fmt::Debug, rc::Rc};

use rand::{rngs::StdRng, Rng as _, SeedableRng as _};

// the implementation of this is basic for prototyping purposes, but it should
// be made to match the behavior of NetLogo Desktop

pub trait NextInt: Debug {
    fn next_int(&mut self, max: i32) -> i32;
}

impl<N: NextInt + ?Sized> NextInt for Rc<RefCell<N>> {
    fn next_int(&mut self, max: i32) -> i32 {
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
    fn next_int(&mut self, max: i32) -> i32 {
        self.rng.gen_range(0..max)
    }
}
