use std::{collections::HashMap, marker::PhantomData};

use crate::exec::CanonExecutionContext;
use lir::HostFunction as Hf;

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
    ) -> Result<(HashMap<lir::FunctionId, JitEntrypoint>, Vec<u8>), (InstallLirError, Vec<u8>)>;

    const HOST_FUNCTION_TABLE: HostFunctionTable;
}

// TODO(wishlist) currently we have hardcoded constants to define what the
// indices of all parameters are for entrypoints and callbacks, i.e. functions
// with known signatures. There must be a cleaner way to get ahold of these
// constants.

#[repr(transparent)]
pub struct JitEntrypoint {
    // TODO(wishlist) for type safety, we probably want to use a newtype
    // over *mut u8 to indicate dynamically passed arguments
    fn_ptr: extern "C" fn(&mut CanonExecutionContext, *mut u8),
}

impl JitEntrypoint {
    /// The index of the context parameter when calling an entrypoint function.
    pub const PARAM_CONTEXT_IDX: usize = 0;
    /// The index of the arguments parameter when calling an entrypoint function.
    pub const PARAM_ARGS_IDX: usize = 1;

    pub fn new(fn_ptr: extern "C" fn(&mut CanonExecutionContext, *mut u8)) -> Self {
        Self { fn_ptr }
    }

    pub fn call(&self, context: &mut CanonExecutionContext, args: *mut u8) {
        (self.fn_ptr)(context, args)
    }
}

#[repr(C)]
pub struct JitCallback<'env, Arg, Ret> {
    pub env: *mut u8,
    pub fn_ptr: extern "C" fn(*mut u8, &mut CanonExecutionContext, Arg) -> Ret,
    pub _phantom: PhantomData<&'env mut ()>,
}

impl<'env, Arg, Ret> JitCallback<'env, Arg, Ret> {
    /// The index of the environment parameter when calling a callback function.
    pub const PARAM_ENV_IDX: usize = 0;
    /// The index of the context parameter when calling a callback function.
    pub const PARAM_CONTEXT_IDX: usize = 1;
    /// The index of the arguments parameter when calling a callback function.
    pub const PARAM_ARG_IDX: usize = 2;

    pub fn call_mut(&mut self, context: &mut CanonExecutionContext, arg: Arg) -> Ret {
        (self.fn_ptr)(self.env, context, arg)
    }
}

/// A hard-coded table of all host function that the engine needs to generate
/// the proper calls from LIR.
pub struct HostFunctionTable {
    pub clear_all: Hf,
    pub reset_ticks: Hf,
    pub advance_tick: Hf,
    pub get_tick: Hf,
    pub create_turtles: Hf,
    pub ask_all_turtles: Hf,
    pub ask_all_patches: Hf,
    pub euclidean_distance_no_wrap: Hf,
    pub list_new: Hf,
    pub list_push: Hf,
    pub one_of_list: Hf,
    pub scale_color: Hf,
    pub rotate_turtle: Hf,
    pub turtle_forward: Hf,
    pub patch_at: Hf,
    pub random_int: Hf,
    pub dynbox_binary_op: Hf,
    pub dynbox_bool_binary_op: Hf,
    pub patch_ahead: Hf,
    pub patch_right_and_ahead: Hf,
    pub diffuse_8_single_variable_buffer: Hf,
}
