use crate::sim::world::World;

#[inline(never)]
pub fn reset_ticks(world: &World) {
    world.tick_counter.reset();
}

#[inline(never)]
pub fn advance_tick(world: &World) {
    if !world.tick_counter.advance() {
        panic!("runtime error: ticks were cleared");
    }
}
