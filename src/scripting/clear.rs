use crate::updater::WriteUpdate;

use super::ExecutionContext;

pub fn clear_all<U: WriteUpdate>(context: &mut ExecutionContext<'_, U>) {
    context.world.clear_all();
}
