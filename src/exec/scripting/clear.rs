use crate::exec::CanonExecutionContext;

#[no_mangle]
#[inline(never)]
pub extern "C" fn clear_all(context: &mut CanonExecutionContext) {
    context.workspace.world.clear_all();
}
