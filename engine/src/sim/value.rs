//! NetLogo runtime values.

pub mod agentset;
mod boolean;
mod dynbox;
mod float;
mod string;
mod r#type;

pub use boolean::NlBool;
pub use dynbox::DynBox;
pub use dynbox::UnpackedDynBox;
pub use float::NlFloat;
pub use string::NlString;
pub use r#type::NlMachineTy;

// TODO(mvp) add box-like representation for indirect values.
// Values such as strings, lists, anonymous procedures, etc. should use this,
// replacing the existing String file
