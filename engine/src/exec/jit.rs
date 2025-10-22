use std::{collections::HashMap, marker::PhantomData, sync::LazyLock};

use lir::typed_index_collections::TiVec;

use crate::{exec::CanonExecutionContext, mir::HostFunctionIds};

pub enum InstallLirError {
    /// Installer state was corrupted and cannot be used to install new
    /// functions.
    InstallerPoisoned,
    /// The runtime encountered an error while instantiating the new functions.
    /// We have no further information.
    RuntimeError,
}

pub trait InstallLir {
    /// Installs the specified LIR program into the current instance. Potential
    /// callbacks and entrypoints in the LIR program are also installed in the
    /// function table, and must have the correct signature.
    ///
    /// # Safety
    ///
    /// The LIR program being installed will use the same namespace and
    /// indirect function table as the current instance. The code must not
    /// cause undefined behavior.
    unsafe fn install_lir(
        lir: &lir::Program,
    ) -> Result<HashMap<lir::FunctionId, JitEntrypoint>, InstallLirError>;
}

#[repr(transparent)]
pub struct JitEntrypoint {
    // TODO for type safety, we probably want to use a newtype over *mut u8 to
    // indicate dynamically passed arguments
    fn_ptr: extern "C" fn(&mut CanonExecutionContext, *mut u8),
}

impl JitEntrypoint {
    pub fn new(fn_ptr: extern "C" fn(&mut CanonExecutionContext, *mut u8)) -> Self {
        Self { fn_ptr }
    }

    pub fn call(&self, context: &mut CanonExecutionContext, args: *mut u8) {
        (self.fn_ptr)(context, args)
    }
}

#[repr(C)]
pub struct JitCallback<'env, Arg, Ret> {
    pub fn_ptr: extern "C" fn(*mut u8, &mut CanonExecutionContext, Arg) -> Ret,
    pub env: *mut u8,
    pub _phantom: PhantomData<&'env ()>,
}

impl<'env, Arg, Ret> JitCallback<'env, Arg, Ret> {
    pub fn call_mut(&mut self, context: &mut CanonExecutionContext, arg: Arg) -> Ret {
        (self.fn_ptr)(self.env, context, arg)
    }
}

// TODO this should be automatically generated from the signatures of the
// actual host functions (probably done from the main crate rather than the
// engine crate)
pub static HOST_FUNCTIONS: LazyLock<(
    TiVec<lir::HostFunctionId, lir::HostFunction>,
    HostFunctionIds,
)> = LazyLock::new(|| {
    let mut host_functions = TiVec::new();
    let clear_all = host_functions.push_and_get_key(lir::HostFunction {
        name: "clear_all",
        parameter_types: vec![lir::ValType::Ptr],
        return_type: vec![],
    });
    let reset_ticks = host_functions.push_and_get_key(lir::HostFunction {
        name: "reset_ticks",
        parameter_types: vec![lir::ValType::Ptr],
        return_type: vec![],
    });
    let create_turtles = host_functions.push_and_get_key(lir::HostFunction {
        name: "create_turtles",
        parameter_types: vec![
            // TODO recheck the types
            lir::ValType::Ptr,
            lir::ValType::I32,
            lir::ValType::I32,
            lir::ValType::Ptr,
            lir::ValType::Ptr,
        ],
        return_type: vec![],
    });
    let ask_all_turtles = host_functions.push_and_get_key(lir::HostFunction {
        name: "for_all_turtles",
        parameter_types: vec![lir::ValType::Ptr, lir::ValType::Ptr, lir::ValType::FnPtr],
        return_type: vec![],
    });
    (host_functions, HostFunctionIds { clear_all, reset_ticks, create_turtles, ask_all_turtles })
});
