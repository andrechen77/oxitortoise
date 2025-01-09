use crate::sim::world::World;

#[no_mangle]
#[inline(never)]
pub extern "C" fn reset_ticks(world: &World) {
    world.tick_counter.reset();
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn advance_tick(world: &World) {
    if !world.tick_counter.advance() {
        panic!("runtime error: ticks were cleared");
    }
}
