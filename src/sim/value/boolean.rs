use derive_more::derive::From;

#[derive(Debug, Clone, Copy, From, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Boolean(pub bool);
