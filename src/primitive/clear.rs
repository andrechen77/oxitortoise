use crate::updater::Update;

use super::ExecutionContext;

pub fn clear_all<'w, U: Update>(context: &mut ExecutionContext<'w, U>) {
    context.world.clear_all();
}
