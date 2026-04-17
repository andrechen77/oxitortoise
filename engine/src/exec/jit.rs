use std::{marker::PhantomData, mem::offset_of};

use crate::{
    hir::HirToMirFnBuilder,
    mir,
    sim::value::PackedAny,
    util::{reflection::Reflect, rng::CanonRng},
    workspace::Workspace,
};
use macro_reflect::{MirReflect, reflect};

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
}

pub trait InstalledObj {
    fn entrypoint(&self, fn_id: lir::FunctionId) -> JitEntrypoint<'_>;
}

// TODO(wishlist) currently we have hardcoded constants to define what the
// indices of all parameters are for entrypoints and callbacks, i.e. functions
// with known signatures. There must be a cleaner way to get ahold of these
// constants.

#[derive(Clone, Copy)]
#[repr(transparent)]
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
    fn_ptr: unsafe extern "C" fn(&mut Workspace, &mut CanonRng, *mut PackedAny, u32),
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
        fn_ptr: unsafe extern "C" fn(&mut Workspace, &mut CanonRng, *mut PackedAny, u32),
    ) -> Self {
        Self { fn_ptr, _lifetime: PhantomData }
    }

    pub fn call(&self, workspace: &mut Workspace, rng: &mut CanonRng, mut args: Vec<PackedAny>) {
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
            (self.fn_ptr)(workspace, rng, args_ptr, args_len);
        }
    }
}

/// A callback that can be called multiple times. Equivalent to FnMut.
#[derive(MirReflect)]
#[repr(C)]
pub struct JitCallback<'env, Arg, Ret> {
    #[mir_accessible(unchecked_type)]
    env: *mut u8,
    #[mir_accessible(unchecked_type)]
    call: unsafe extern "C" fn(*mut u8, &mut Workspace, &mut CanonRng, Arg) -> Ret,
    #[mir_accessible(unchecked_type)]
    drop: unsafe extern "C" fn(*mut u8),
    _phantom: PhantomData<&'env mut ()>,
}

#[reflect]
impl<'env> Reflect for JitCallback<'static, crate::sim::turtle::TurtleId, ()> {}

#[reflect]
impl<'env> Reflect for JitCallback<'static, crate::sim::patch::PatchId, ()> {}

impl<'env, Arg, Ret> JitCallback<'env, Arg, Ret> {
    /// # Safety
    ///
    /// The `call` and `drop` functions must be safe to call as long as the
    /// lifetime `'env` is live for the duration of the call, and assuming the
    /// env pointer being passed during the call is the same as the one passed
    /// to this function, and that the `drop` function has not been called yet.
    /// It must be safe to call the `call` function multiple times.
    pub unsafe fn new(
        env: *mut u8,
        call: extern "C" fn(*mut u8, &mut Workspace, &mut CanonRng, Arg) -> Ret,
        drop: extern "C" fn(*mut u8),
    ) -> Self {
        Self { env, call, drop, _phantom: PhantomData }
    }

    pub fn call_mut(&mut self, workspace: &mut Workspace, rng: &mut CanonRng, arg: Arg) -> Ret {
        // SAFETY: we are passing the same env pointer to the function call as
        // the one passed during the construction of the JitCallback. In
        // addition, because we take &mut Self where Self has lifetime 'env, we
        // also know that the lifetime of 'env is live for the duration of the
        // call. The `drop` function has not been called yet.
        unsafe { (self.call)(self.env, workspace, rng, arg) }
    }
}

impl<'env, Arg, Ret> JitCallback<'env, Arg, Ret>
where
    Self: Reflect,
{
    pub fn mir_initialize(
        builder: &mut HirToMirFnBuilder,
        env: mir::Place,
        call_fn: mir::FunctionId,
        drop_fn: mir::FunctionId,
    ) -> mir::LocalId {
        // this will light up if we change the fields without updating the function.
        // This helps us ensure that all fields are initialized
        #[allow(dead_code)]
        const {
            let _ = |s: JitCallback<'env, Arg, Ret>| {
                let JitCallback::<'env, Arg, Ret> { env: _, call: _, drop: _, _phantom: _ } = s;
            };
        };

        let result_local =
            builder.mir.create_local(mir::LocalDecl { debug_name: None, ty: Self::mir_type() });

        let call_fn =
            builder.mir.add_operation(None, mir::Operation::FunctionPtr { function: call_fn });
        let drop_fn =
            builder.mir.add_operation(None, mir::Operation::FunctionPtr { function: drop_fn });

        builder.mir.add_operation_with_dst(
            result_local.place().proj_field(offset_of!(Self, env)),
            mir::Operation::Operand(mir::PlaceOperand::Copy(env)),
        );
        builder.mir.add_operation_with_dst(
            result_local.place().proj_field(offset_of!(Self, call)),
            mir::Operation::Operand(mir::PlaceOperand::Move(call_fn)),
        );
        builder.mir.add_operation_with_dst(
            result_local.place().proj_field(offset_of!(Self, drop)),
            mir::Operation::Operand(mir::PlaceOperand::Move(drop_fn)),
        );

        builder.mir.set_as_init(result_local);
        result_local
    }
}

impl<'env, Arg, Ret> Drop for JitCallback<'env, Arg, Ret> {
    fn drop(&mut self) {
        // SAFETY: we are passing the same env pointer to the function call as
        // the one passed during the construction of the JitCallback. In
        // addition, because we take &mut Self where Self has lifetime 'env, we
        // also know that the lifetime of 'env is live for the duration of the
        // call. The `drop` function has not been called yet as this is the only
        // time the function is ever called.
        unsafe { (self.drop)(self.env) };
    }
}
