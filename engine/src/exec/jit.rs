use std::{collections::HashMap, marker::PhantomData};

use crate::exec::CanonExecutionContext;

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
    pub fn_ptr: extern "C" fn(*mut u8, &mut CanonExecutionContext, Arg) -> Ret,
    pub env: *mut u8,
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

// TODO(mvp) this should be automatically generated from the signatures
// of the actual host functions (probably done from the main crate rather than
// the engine crate).
//
// TODO(mvp_ants) once the compiler pipeline is done, double-check that the
// signatures match.
#[rustfmt::skip] // keep struct definitions over multiple lines
pub mod host_fn {
    use lir::HostFunction as Hf;
    use lir::ValType::{I16, F64, FnPtr, Ptr};

    pub const CLEAR_ALL: &Hf = &Hf {
        name: "clear_all",
        parameter_types: &[Ptr],
        return_type: &[],
    };

    pub const RESET_TICKS: &Hf = &Hf {
        name: "reset_ticks",
        parameter_types: &[Ptr],
        return_type: &[],
    };

    pub const CREATE_TURTLES: &Hf = &Hf {
        name: "reset_ticks",
        parameter_types: &[Ptr],
        return_type: &[],
    };

    pub const ASK_ALL_TURTLES: &Hf = &Hf {
        name: "for_all_turtles",
        parameter_types: &[Ptr, Ptr, FnPtr],
        return_type: &[],
    };

    pub const ASK_ALL_PATCHES: &Hf = &Hf {
        name: "for_all_patches",
        parameter_types: &[Ptr, Ptr, FnPtr],
        return_type: &[],
    };

    pub const EUCLIDEAN_DISTANCE_NO_WRAP: &Hf = &Hf {
        name: "distance_euclidean_no_wrap",
        parameter_types: &[F64, F64, F64, F64],
        return_type: &[F64],
    };

    // fn list_new() -> NlBox<NlList>
    pub const LIST_NEW: &Hf = &Hf {
        name: "list_new",
        parameter_types: &[],
        return_type: &[Ptr],
    };

    // fn list_push(list: NlBox<NlList>, element: DynBox) -> NlBox<NlList>
    pub const LIST_PUSH: &Hf = &Hf {
        name: "list_push",
        parameter_types: &[Ptr, F64],
        return_type: &[Ptr]
    };

    // fn one_of_list(context: &mut CanonExecutionContext, list: NlBox<NlList>) -> DynBox
    pub const ONE_OF_LIST: &Hf = &Hf {
        name: "one_of_list",
        parameter_types: &[Ptr, Ptr],
        return_type: &[Ptr],
    };

    // fn scale_color(color: Color, number: NlFloat, range1: NlFloat, range2: NlFloat) -> Color
    pub const SCALE_COLOR: &Hf = &Hf {
        name: "scale_color",
        parameter_types: &[F64, F64, F64, F64],
        return_type: &[F64],
    };

    // fn diffuse_8_single_variable_buffer(ctx: &mut CanonExecutionContext, field: AgentFieldDescriptor, fraction: NlFloat)
    pub const DIFFUSE_8_SINGLE_VARIABLE_BUFFER: &Hf = &Hf {
        name: "diffuse_8_single_variable_buffer",
        parameter_types: &[Ptr, I16, F64],
        return_type: &[],
    };
}
