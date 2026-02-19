use std::marker::PhantomData;

use crate::{exec::CanonExecutionContext, sim::value::PackedAny};
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
    type Obj: InstalledObj;

    /// Installs the specified LIR program into the current instance. Potential
    /// callbacks and entrypoints in the LIR program are also installed in the
    /// function table, and must have the correct signature.
    ///
    /// # Safety
    ///
    /// The LIR program being installed will use the same namespace and
    /// indirect function table as the current instance. The code must not
    /// cause undefined behavior when the functions are called according to the
    /// safety requirements articulated in the following.
    /// - [`JitEntrypoint::new`]
    /// - [`JitCallback::new`]
    unsafe fn install_lir(&mut self, lir: &lir::Program) -> Result<Self::Obj, InstallLirError>;

    const HOST_FUNCTION_TABLE: HostFunctionTable;
}

pub trait InstalledObj {
    fn entrypoint(&self, fn_id: lir::FunctionId) -> JitEntrypoint<'_>;
}

// TODO(wishlist) currently we have hardcoded constants to define what the
// indices of all parameters are for entrypoints and callbacks, i.e. functions
// with known signatures. There must be a cleaner way to get ahold of these
// constants.

#[derive(Clone, Copy)]
pub struct JitEntrypoint<'a> {
    /// A pointer to a shim that can take dynamic arguments. The second
    /// parameter is a pointer to a dynamically-sized owned array of arguments,
    /// where the third parameter is the length of the array. This function will
    /// move every argument out of the array (so be careful to avoid double
    /// dropping); note that this does not deallocate the memory for the array
    /// itself.
    ///
    /// # Safety
    ///
    /// See [`JitEntrypoint::new`] method for the safety requirements on the
    /// caller of this function.
    fn_ptr: unsafe extern "C" fn(&mut CanonExecutionContext, *mut PackedAny, u32),
    _lifetime: PhantomData<&'a mut ()>,
}

impl<'a> JitEntrypoint<'a> {
    /// # Safety
    ///
    /// The function pointer must be safe to call if the following requirements
    /// are met by the caller. The arguments pointer and length must correspond
    /// to a valid array of PackedAny values, and the caller will treat those
    /// pointed-to values as being moved out of the array by the function. In
    /// addition, the lifetime of `'a` is live for the duration of the call.
    /// There are no other safety requirements to call this function
    pub unsafe fn new(
        fn_ptr: unsafe extern "C" fn(&mut CanonExecutionContext, *mut PackedAny, u32),
    ) -> Self {
        Self { fn_ptr, _lifetime: PhantomData }
    }

    pub fn call(&self, context: &mut CanonExecutionContext, mut args: Vec<PackedAny>) {
        let args_len = u32::try_from(args.len()).unwrap();
        let args_ptr = args.as_mut_ptr();
        // SAFETY: the arguments pointer and length come from a Vec<PackedAny>
        // which was not modified at all besides set_len(0). By doing
        // set_len(0), we assume that the values were moved out of the array by
        // the function call and prevent double dropping, even if the function
        // panics. Because we take &Self where Self has lifetime 'a, we also
        // know that the lifetime of 'a is live for the duration of the call.
        unsafe {
            args.set_len(0);
            (self.fn_ptr)(context, args_ptr, args_len);
        }
    }
}

/// A callback that can be called multiple times. Equivalent to FnMut.
#[repr(C)]
pub struct JitCallback<'env, Arg, Ret> {
    env: *mut u8,
    fn_ptr: unsafe extern "C" fn(*mut u8, &mut CanonExecutionContext, Arg) -> Ret,
    _phantom: PhantomData<&'env mut ()>,
}

impl<'env, Arg, Ret> JitCallback<'env, Arg, Ret> {
    /// # Safety
    ///
    /// The function pointer itself must be safe to call as long as the lifetime
    /// `'env` is live for the duration of the call, and assuming the env
    /// pointer being passed during the call is the same as the one passed to
    /// this function. The function pointer may be called multiple times.
    pub unsafe fn new(
        env: *mut u8,
        fn_ptr: extern "C" fn(*mut u8, &mut CanonExecutionContext, Arg) -> Ret,
    ) -> Self {
        Self { env, fn_ptr, _phantom: PhantomData }
    }

    pub fn call_mut(&mut self, context: &mut CanonExecutionContext, arg: Arg) -> Ret {
        // SAFETY: we are passing the same env pointer to the function call as
        // the one passed during the construction of the JitCallback. In
        // addition, because we take &mut Self where Self has lifetime 'env, we
        // also know that the lifetime of 'env is live for the duration of the
        // call.
        unsafe { (self.fn_ptr)(self.env, context, arg) }
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
    pub any_binary_op: Hf,
    pub any_bool_binary_op: Hf,
    pub patch_ahead: Hf,
    pub patch_right_and_ahead: Hf,
    pub diffuse_8_single_variable_buffer: Hf,
}
