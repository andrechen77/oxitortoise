// for the row buffer
#![feature(slice_ptr_get, alloc_layout_extra, const_type_name)]

// when compiling to wasm, you must pass -Zwasm-c-abi=spec to rustc for correct
// compatibility with Emscripten-emitted Wasm. Add this to .cargo/config.toml or
// pass via RUSTFLAGS.

// TODO in the future when we actually generate our own JIT
// code instead of relying on Emscripten, it doesn't actually matter what the
// ABI is, as long as it is known and stable.

pub mod exec;
pub mod sim;
pub mod updater;
pub mod util;
pub mod workspace;

// pub use exec::scripting::prelude as scripting_prelude;
