use derive_more::derive::{From, Not};

#[derive(Debug, Clone, Copy, From, PartialEq, Eq, PartialOrd, Ord, Not)]
#[repr(transparent)]
pub struct Boolean(pub bool);
