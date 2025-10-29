// #![feature(gen_blocks, step_trait)]

pub extern crate lir;

mod codegen;
mod stackify_generic;
mod stackify_lir;

pub use codegen::{FnTableSlotAllocator, lir_to_wasm};
