use crate::{updater::WriteUpdate, util::rng::Rng};

use super::ExecutionContext;

#[inline(never)]
pub fn random<U: WriteUpdate>(context: &mut ExecutionContext<'_, U>, range: i64) -> i64 {
    let mut rng = context.next_int.borrow_mut();
    match range {
        0 => 0,
        r if r < 0 => -rng.next_int(-r),
        r => rng.next_int(r),
    }
}
