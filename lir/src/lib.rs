#![feature(gen_blocks, step_trait)]
pub mod lir;
pub use lir::*;
// mod stackify;
mod stackify;
pub mod wasm;

// Reexports so that the macros can use them.
pub mod reexports;
