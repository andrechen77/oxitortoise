use std::{ops::Deref, sync::Mutex};

use macro_reflect::{ReflectComponents, reflect};

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

#[derive(Debug, ReflectComponents)]
pub struct CanonRng(mersenne_twister::MersenneTwister);

#[reflect]
impl Reflect for CanonRng {}

impl Rng for CanonRng {
    fn next_int(&mut self, max: i64) -> i64 {
        self.0.next_int(max)
    }
}

#[reflect]
impl Reflect for &mut CanonRng {}
