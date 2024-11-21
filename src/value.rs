//! NetLogo runtime values.

mod boolean;
mod float;
mod string;
mod polyvalue;

pub use boolean::Boolean;
pub use float::Float;
pub use string::String;
pub use polyvalue::{PolyValue, Type};

// TODO add other types such as link references, agentsets, lists, reporters,
// commmands
