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
pub use r#type::NetlogoInternalType;
pub use string::String;

// TODO add other types such as link references, lists, reporters,
// commmands
