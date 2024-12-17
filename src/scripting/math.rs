use crate::{updater::WriteUpdate, util::rng::NextInt};

use super::ExecutionContext;

pub fn random<U: WriteUpdate>(context: &mut ExecutionContext<'_, U>, range: i64) -> i64 {
    let mut rng = context.next_int.borrow_mut();
    if range < 0 {
        -rng.next_int(-range)
    } else {
        rng.next_int(range)
    }
}
