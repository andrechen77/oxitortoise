use derive_more::derive::From;

#[derive(Debug, Clone, Copy, From, PartialEq, Eq, PartialOrd, Ord)]
pub struct Boolean(pub bool);
