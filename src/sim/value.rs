//! NetLogo runtime values.

pub mod agentset;
mod boolean;
mod float;
mod polyvalue;
mod string;

pub use boolean::Boolean;
pub use float::Float;
pub use polyvalue::PolyValue;
pub use string::String;

// TODO add other types such as link references, lists, reporters,
// commmands
