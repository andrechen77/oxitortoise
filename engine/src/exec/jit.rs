use std::collections::HashMap;

use crate::exec::CanonExecutionContext;

pub trait InstallLir {
    unsafe fn install_lir(
        lir: &lir::Program,
    ) -> Result<HashMap<lir::FunctionId, JitEntrypoint>, ()>;
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
pub struct JitCallback<Arg, Ret> {
    pub fn_ptr: extern "C" fn(*mut u8, &mut CanonExecutionContext, Arg) -> Ret,
    pub env: *mut u8,
}

impl<Arg, Ret> JitCallback<Arg, Ret> {
    pub fn call_mut(&mut self, context: &mut CanonExecutionContext, arg: Arg) -> Ret {
        (self.fn_ptr)(self.env, context, arg)
    }
}
