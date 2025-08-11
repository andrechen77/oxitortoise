use crate::exec::CanonExecutionContext;

#[cfg(target_arch = "wasm32")]
mod wasm;

// TODO remove this once all JIT code goes through the LIR
#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[repr(transparent)]
pub struct JitEntry {
    // TODO for type safety, we probably want to use a newtype over *mut u8 to
    // indicate dynamically passed arguments
    fn_ptr: extern "C" fn(&mut CanonExecutionContext, *mut u8),
}

impl JitEntry {
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

// I prefer keeping the calls explicit. it should be elided during optimization

// impl<Arg, Ret> FnOnce<(&mut CanonExecutionContext<'_>, Arg)> for JitCallback<Arg, Ret> {
//     type Output = Ret;
//     extern "rust-call" fn call_once(
//         self,
//         (context, arg): (&mut CanonExecutionContext<'_>, Arg),
//     ) -> Self::Output {
//         (self.fn_ptr)(context, self.env, arg)
//     }
// }

// impl<Arg, Ret> FnMut<(&mut CanonExecutionContext<'_>, Arg)> for JitCallback<Arg, Ret> {
//     extern "rust-call" fn call_mut(
//         &mut self,
//         (context, arg): (&mut CanonExecutionContext<'_>, Arg),
//     ) -> Ret {
//         (self.fn_ptr)(context, self.env, arg)
//     }
// }
