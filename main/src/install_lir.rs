use engine::{
    exec::jit::{InstalledObj, JitEntrypoint},
    lir,
    sim::value::PackedAny,
    util::rng::CanonRng,
    workspace::Workspace,
};

use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::LirInstaller;

#[cfg(not(target_arch = "wasm32"))]
mod cranelift;
#[cfg(not(target_arch = "wasm32"))]
pub use cranelift::LirInstaller;

pub struct Obj {
    entrypoints: HashMap<
        lir::FunctionId,
        unsafe extern "C" fn(&mut Workspace, &mut CanonRng, *mut PackedAny, u32),
    >,
}

impl InstalledObj for Obj {
    fn entrypoint(&self, fn_id: lir::FunctionId) -> JitEntrypoint<'_> {
        let fn_ptr = self.entrypoints[&fn_id];
        // SAFETY: according to the safety requirements of the
        // [`InstallLir::install_lir`] the entrypoint functions satisfy the
        // safety requirements of `JitEntrypoint::new`, and this is an
        // entrypoint function.
        unsafe { JitEntrypoint::new(fn_ptr) }
    }
}
