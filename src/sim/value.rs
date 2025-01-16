//! NetLogo runtime values.

pub mod agentset;
mod boolean;
mod float;
mod string;

pub use boolean::Boolean;
use derive_more::derive::From;
pub use float::{Float, TryAsFloat};
pub use string::String;

#[derive(Debug, Clone, Copy, From)]
pub struct Nobody;

impl From<&()> for &Nobody {
    fn from(_: &()) -> Self {
        &Nobody
    }
}

impl TryAsFloat for Nobody {
    fn try_as_float(&self) -> Option<Float> {
        None
    }
}

// TODO add other types such as link references, lists, reporters,
// commmands
