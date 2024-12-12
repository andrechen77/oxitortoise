use crate::updater::Update;

use super::ExecutionContext;

pub fn clear_all<'w, U: Update>(context: &mut ExecutionContext<'w, U>) {
    context.world.clear_all();
}

pub fn reset_ticks<'w, U: Update>(context: &mut ExecutionContext<'w, U>) {
    context.world.tick_counter.clear();
}
