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
    use lir::ValType::{I8, I16, I32, I64, F64, FnPtr, Ptr};

    pub static CLEAR_ALL: &Hf = &Hf {
        name: "clear_all",
        parameter_types: &[Ptr],
        return_type: &[],
    };

    pub static RESET_TICKS: &Hf = &Hf {
        name: "reset_ticks",
        parameter_types: &[Ptr],
        return_type: &[],
    };

    // fn advance_tick(context: &mut Context)
    pub static ADVANCE_TICK: &Hf = &Hf {
        name: "advance_tick",
        parameter_types: &[Ptr],
        return_type: &[],
    };

    // fn get_tick(context: &mut Context) -> NlFloat
    pub static GET_TICK: &Hf = &Hf {
        name: "get_tick",
        parameter_types: &[Ptr],
        return_type: &[F64],
    };

    pub static CREATE_TURTLES: &Hf = &Hf {
        name: "reset_ticks",
        parameter_types: &[Ptr],
        return_type: &[],
    };

    pub static ASK_ALL_TURTLES: &Hf = &Hf {
        name: "for_all_turtles",
        parameter_types: &[Ptr, Ptr, FnPtr],
        return_type: &[],
    };

    pub static ASK_ALL_PATCHES: &Hf = &Hf {
        name: "for_all_patches",
        parameter_types: &[Ptr, Ptr, FnPtr],
        return_type: &[],
    };

    pub static EUCLIDEAN_DISTANCE_NO_WRAP: &Hf = &Hf {
        name: "distance_euclidean_no_wrap",
        parameter_types: &[F64, F64, F64, F64],
        return_type: &[F64],
    };

    // fn list_new() -> NlBox<NlList>
    pub static LIST_NEW: &Hf = &Hf {
        name: "list_new",
        parameter_types: &[],
        return_type: &[Ptr],
    };

    // fn list_push(list: NlBox<NlList>, element: DynBox) -> NlBox<NlList>
    pub static LIST_PUSH: &Hf = &Hf {
        name: "list_push",
        parameter_types: &[Ptr, F64],
        return_type: &[Ptr]
    };

    // fn one_of_list(context: &mut CanonExecutionContext, list: NlBox<NlList>) -> DynBox
    pub static ONE_OF_LIST: &Hf = &Hf {
        name: "one_of_list",
        parameter_types: &[Ptr, Ptr],
        return_type: &[Ptr],
    };

    // fn scale_color(color: Color, number: NlFloat, range1: NlFloat, range2: NlFloat) -> Color
    pub static SCALE_COLOR: &Hf = &Hf {
        name: "scale_color",
        parameter_types: &[F64, F64, F64, F64],
        return_type: &[F64],
    };

    // fn rotate_turtle(context: &mut CanonExecutionContext, turtle_id: TurtleId, angle: NlFloat)
    pub static ROTATE_TURTLE: &Hf = &Hf {
        name: "rotate_turtle",
        parameter_types: &[Ptr, I64, F64],
        return_type: &[],
    };

    // fn turtle_forward(context: &mut CanonExecutionContext, turtle_id: TurtleId, distance: NlFloat)
    pub static TURTLE_FORWARD: &Hf = &Hf {
        name: "turtle_forward",
        parameter_types: &[Ptr, I64, F64],
        return_type: &[],
    };

    // fn patch_at(context: &mut CanonExecutionContext, point: Point) -> OptionPatchId
    pub static PATCH_AT: &Hf = &Hf {
        name: "patch_at",
        parameter_types: &[Ptr, F64, F64],
        return_type: &[I32],
    };

    // fn random_int(context: &mut CanonExecutionContext, max: NlFloat) -> NlFloat
    pub static RANDOM_INT: &Hf = &Hf {
        name: "random_int",
        parameter_types: &[Ptr, F64],
        return_type: &[F64],
    };

    // fn dynbox_binary_op(lhs: DynBox, rhs: DynBox, op: u8) -> DynBox
    pub static DYNBOX_BINARY_OP: &Hf = &Hf {
        name: "dynbox_binary_op",
        parameter_types: &[F64, F64, I8],
        return_type: &[F64],
    };

    pub static DYNBOX_BOOL_BINARY_OP: &Hf = &Hf {
        name: "dynbox_bool_binary_op",
        parameter_types: &[F64, F64, I8],
        return_type: &[I8],
    };

    // fn patch_ahead(context: &mut CanonExecutionContext, turtle_id: u64, distance: NlFloat) -> PatchId
    pub static PATCH_AHEAD: &Hf = &Hf {
        name: "patch_ahead",
        parameter_types: &[Ptr, I64, F64],
        return_type: &[I32],
    };

    // fn patch_right_and_ahead(context: &mut CanonExecutionContext, turtle_id: u64, distance: NlFloat, angle: NlFloat) -> PatchId
    pub static PATCH_RIGHT_AND_AHEAD: &Hf = &Hf {
        name: "patch_right_and_ahead",
        parameter_types: &[Ptr, I64, F64, F64],
        return_type: &[I32],
    };

    // fn diffuse_8_single_variable_buffer(ctx: &mut CanonExecutionContext, field: AgentFieldDescriptor, fraction: NlFloat)
    pub static DIFFUSE_8_SINGLE_VARIABLE_BUFFER: &Hf = &Hf {
        name: "diffuse_8_single_variable_buffer",
        parameter_types: &[Ptr, I16, F64],
        return_type: &[],
    };
}
