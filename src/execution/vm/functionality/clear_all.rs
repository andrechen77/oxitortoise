use crate::execution::vm::Execute;

#[derive(Debug)]
pub struct ClearAll {}

impl<U> Execute<U> for ClearAll {
    unsafe fn execute<'w>(&self, context: &mut crate::execution::vm::ExecutionContext<'w, U>) {
        context.world.clear_all();
    }
}
