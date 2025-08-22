//! NetLogo runtime values.

pub mod agentset;
mod boolean;
mod dynbox;
mod float;
mod string;
mod r#type;

pub use boolean::Boolean;
pub use dynbox::DynBox;
pub use float::Float;
pub use string::String;
pub use r#type::NetlogoInternalType;

// TODO add other types such as link references, lists, reporters,
// commmands
