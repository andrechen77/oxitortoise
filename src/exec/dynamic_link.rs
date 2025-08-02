use slotmap::{Key as _, KeyData};

use crate::{
    exec::{helpers, jit::JitCallback, CanonExecutionContext},
    sim::{
        agent_schema::AgentFieldDescriptor,
        patch::PatchId,
        topology::{Heading, Point, PointInt},
        turtle::BreedId,
        value::Float,
        world::World,
    },
    util::rng::Rng as _,
};

#[no_mangle]
pub extern "C" fn oxitortoise_is_nan(value: f64) -> bool {
    value.is_nan()
}

#[no_mangle]
pub extern "C" fn oxitortoise_round(value: Float) -> Float {
    // TODO is it possible for this to go from finite to non-finite?
    Float::new(value.get().round())
}

#[no_mangle]
pub extern "C" fn oxitortoise_clear_all(context: &mut CanonExecutionContext) {
    context.workspace.world.clear_all();
}

#[no_mangle]
pub extern "C" fn oxitortoise_reset_ticks(world: &mut World) {
    world.tick_counter.reset();
}

#[no_mangle]
pub extern "C" fn oxitortoise_get_ticks(world: &mut World) -> Float {
    world.tick_counter.get().unwrap() // TODO: handle error
}

#[no_mangle]
pub extern "C" fn oxitortoise_advance_tick(world: &mut World) {
    world.tick_counter.advance().unwrap(); // TODO: handle error
}

#[no_mangle]
pub extern "C" fn oxitortoise_create_turtles(
    context: &mut CanonExecutionContext,
    breed: u64,
    count: u64,
    position: Point,
    mut birth_command: JitCallback<u64, ()>,
) {
    let breed: BreedId = KeyData::from_ffi(breed).into();
    helpers::create_turtles(context, breed, count, position, |context, turtle_id| {
        birth_command.call_mut(context, turtle_id.to_ffi())
    });
}

#[no_mangle]
pub extern "C" fn oxitortoise_for_all_turtles(
    context: &mut CanonExecutionContext,
    mut block: JitCallback<u64, ()>,
) {
    helpers::for_all_turtles(context, |context, turtle_id| {
        block.call_mut(context, turtle_id.to_ffi())
    });
}

#[no_mangle]
pub extern "C" fn oxitortoise_for_all_patches(
    context: &mut CanonExecutionContext,
    mut block: JitCallback<PatchId, ()>,
) {
    helpers::for_all_patches(context, |context, patch_id| {
        block.call_mut(context, patch_id)
    });
}

#[no_mangle]
pub extern "C" fn oxitortoise_distance_euclidean_no_wrap(a: Point, b: Point) -> Float {
    crate::sim::topology::euclidean_distance_no_wrap(a, b).into()
}

#[no_mangle]
pub extern "C" fn oxitortoise_offset_distance_by_heading(
    world: &World,
    point: Point,
    heading: Heading,
    distance: Float,
) -> Point {
    world
        .topology
        .offset_distance_by_heading(point, heading, distance)
        .unwrap_or_else(|| Point {
            x: f64::NAN,
            y: f64::NAN,
        })
}

#[no_mangle]
pub extern "C" fn oxitortoise_patch_at(world: &World, point: PointInt) -> PatchId {
    world.topology.patch_at(point)
}

#[no_mangle]
pub extern "C" fn oxitortoise_normalize_heading(heading: Float) -> Heading {
    heading.into()
}

#[no_mangle]
pub extern "C" fn oxitortoise_diffuse_8(
    world: &mut World,
    field: AgentFieldDescriptor,
    diffusion_rate: Float,
) {
    crate::sim::topology::diffuse::diffuse_8_single_variable_buffer(world, field, diffusion_rate);
}

#[no_mangle]
pub extern "C" fn oxitortoise_scale_color(
    color: Float,
    value: Float,
    min: Float,
    max: Float,
) -> Float {
    crate::sim::color::scale_color(color.into(), value, min, max).into()
}

#[no_mangle]
pub extern "C" fn oxitortoise_next_int(context: &mut CanonExecutionContext, max: u32) -> u32 {
    context.next_int.next_int(max as i64) as u32
}

#[no_mangle]
pub extern "C" fn oxitortoise_get_default_turtle_breed(context: &mut CanonExecutionContext) -> u64 {
    let breed_id = context
        .workspace
        .world
        .turtles
        .breeds()
        .keys()
        .next()
        .unwrap();
    breed_id.data().as_ffi()
}
