use crate::updater::Update;

use super::ExecutionContext;

pub fn clear_all<U: Update>(context: &mut ExecutionContext<'_, U>) {
    context.world.clear_all();
}
