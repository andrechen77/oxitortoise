use crate::{exec::CanonExecutionContext, util::rng::Rng as _};

#[no_mangle]
#[inline(never)]
pub extern "C" fn random(context: &mut CanonExecutionContext, range: i64) -> i64 {
    let mut rng = context.next_int.borrow_mut();
    match range {
        0 => 0,
        r if r < 0 => -rng.next_int(-r),
        r => rng.next_int(r),
    }
}
