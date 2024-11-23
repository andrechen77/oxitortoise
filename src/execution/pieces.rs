//! The building blocks of a NetLogo program.
//!
//! These implement either [`super::opaque::ExecuteOpaque`] or (TODO the non-opaque ones lol)

mod compound_statement;
mod clear_all;
mod create_turtles;

pub use compound_statement::CompoundStatementMonolith;
pub use clear_all::ClearAllMonolith;
pub use create_turtles::CreateTurtlesMonolith;
