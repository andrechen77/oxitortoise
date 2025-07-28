use crate::exec::CanonExecutionContext;

#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

pub struct JitFn {
    fn_ptr: extern "C" fn(*mut CanonExecutionContext, *mut u8),
}

impl JitFn {
    pub fn call(&self, context: &mut CanonExecutionContext, args: *mut u8) {
        (self.fn_ptr)(context, args)
    }
}
