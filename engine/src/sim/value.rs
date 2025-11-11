//! NetLogo runtime values.

pub mod agentset;
mod boolean;
mod dynbox;
mod float;
mod string;
mod r#type;

pub use boolean::Boolean;
pub use dynbox::DynBox;
pub use dynbox::UnpackedDynBox;
pub use float::Float;
pub use string::String;
pub use r#type::NetlogoMachineType;

// TODO(mvp) add box-like representation for indirect values.
// Values such as strings, lists, anonymous procedures, etc. should use this,
// replacing the existing String file
