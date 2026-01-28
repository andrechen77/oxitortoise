use std::collections::HashMap;

use engine::{
    exec::jit::{InstallLir, InstallLirError, JitEntrypoint},
    lir,
};

mod export_interface;
mod install_lir;

pub struct LirInstaller;

impl InstallLir for LirInstaller {
    unsafe fn install_lir(
        lir: &lir::Program,
    ) -> Result<HashMap<lir::FunctionId, JitEntrypoint>, InstallLirError> {
        unsafe { install_lir::install_lir(lir) }
    }
}
