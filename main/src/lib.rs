// when compiling to wasm, you must pass -Zwasm-c-abi=spec to rustc for correct
// compatibility with Emscripten-emitted Wasm. Add this to .cargo/config.toml or
// pass via RUSTFLAGS.

// TODO in the future when we actually generate our own JIT
// code instead of relying on Emscripten, it doesn't actually matter what the
// ABI is, as long as it is known and stable.

use std::collections::HashMap;

use engine::{
    exec::jit::{InstallLir, JitEntrypoint},
    lir,
};

mod export_interface;
mod install_lir;

pub struct LirInstaller;

impl InstallLir for LirInstaller {
    unsafe fn install_lir(
        lir: &lir::Program,
    ) -> Result<HashMap<lir::FunctionId, JitEntrypoint>, ()> {
        unsafe { install_lir::install_lir(lir) }
    }
}
