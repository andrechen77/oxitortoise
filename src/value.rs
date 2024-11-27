//! NetLogo runtime values.

mod boolean;
mod float;
mod polyvalue;
mod string;

pub use boolean::Boolean;
pub use float::Float;
pub use polyvalue::{ContainedInValue, PolyValue, Type};
pub use string::String;

// TODO add other types such as link references, agentsets, lists, reporters,
// commmands
