use derive_more::derive::{From, Not};

use super::{Float, TryAsFloat};

#[derive(Debug, Clone, Copy, From, PartialEq, Eq, PartialOrd, Ord, Not)]
#[repr(transparent)]
pub struct Boolean(pub bool);

impl TryAsFloat for Boolean {
    fn try_as_float(&self) -> Option<Float> {
        None
    }
}
