use engine::{
    exec::jit::JitEntrypoint, reflection::mir, sim::value::PackedAny, util::rng::CanonRng,
    workspace::Workspace,
};

use std::collections::BTreeMap;

use crate::install_lir::LirInstaller;

pub enum InstallMirError {
    /// Installer state was corrupted and cannot be used to install new
    /// functions.
    InstallerPoisoned,
    /// The runtime encountered an error while instantiating the new functions.
    /// We have no further information.
    RuntimeError,
}

#[derive(Default)]
pub struct MirInstaller {
    lir: LirInstaller,
}

impl MirInstaller {
    /// Installs the specified MIR program into the current instance. Potential
    /// callbacks and entrypoints in the MIR program are also installed in the
    /// function table, and must have the correct signature.
    ///
    /// # Safety
    ///
    /// The MIR program being installed will use the same namespace and
    /// indirect function table as the current instance. The code must not
    /// cause undefined behavior when the functions are called according to the
    /// safety requirements articulated in the following.
    /// - [`JitEntrypoint::new`]
    /// - [`JitCallback::new`]
    unsafe fn install_mir(
        &mut self,
        _program: &mir::Program,
    ) -> Result<InstalledObj, InstallMirError> {
        todo!("TODO");
    }
}

pub struct InstalledObj {
    entrypoints: BTreeMap<
        mir::FunctionId,
        unsafe extern "C" fn(&mut Workspace, &mut CanonRng, *mut PackedAny, u32),
    >,
}

impl InstalledObj {
    pub fn entrypoint(&self, fn_id: mir::FunctionId) -> JitEntrypoint<'_> {
        let fn_ptr = self.entrypoints[&fn_id];
        // SAFETY: according to the safety requirements of the
        // [`InstallMir::install_mir`] the entrypoint functions satisfy the
        // safety requirements of `JitEntrypoint::new`, and this is an
        // entrypoint function.
        unsafe { JitEntrypoint::new(fn_ptr) }
    }
}
