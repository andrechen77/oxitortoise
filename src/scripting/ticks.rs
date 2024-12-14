use crate::sim::world::World;

pub fn reset_ticks(world: &World) {
    world.tick_counter.reset();
}

pub fn advance_tick(world: &World) {
    if !world.tick_counter.advance() {
        panic!("runtime error: ticks were cleared");
    }
}
