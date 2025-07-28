use flagset::FlagSet;
use slotmap::{Key as _, KeyData};

use crate::{
    exec::CanonExecutionContext,
    sim::{
        agent_schema::AgentFieldDescriptor,
        patch::PatchId,
        tick::Tick,
        topology::{Heading, Point, PointInt},
        turtle::{BreedId, TurtleId},
        value::{
            agentset::{AllPatches, AllTurtles, PatchIterator, TurtleIterator},
            Float,
        },
        world::World,
    },
    updater::{CanonUpdater, WriteUpdate as _},
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
pub extern "C" fn oxitortoise_update_turtle(
    updater: &mut CanonUpdater,
    world: &World,
    turtle_id: TurtleId,
    flags: u16,
) {
    updater.update_turtle(world, turtle_id, FlagSet::new(flags).unwrap()); // TODO once confident this should become unsafe
}

#[no_mangle]
pub extern "C" fn oxitortoise_update_patch(
    updater: &mut CanonUpdater,
    world: &World,
    patch_id: PatchId,
    flags: u8,
) {
    updater.update_patch(world, patch_id, FlagSet::new(flags).unwrap()); // TODO once confident this should become unsafe
}

#[no_mangle]
pub extern "C" fn oxitortoise_update_tick(updater: &mut CanonUpdater, tick: Tick) {
    updater.update_tick(tick);
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
) -> *mut TurtleIterator {
    let breed: BreedId = KeyData::from_ffi(breed).into();
    let created_turtles = context.workspace.world.turtles.create_turtles(
        breed,
        count,
        position,
        &mut context.next_int,
    );
    Box::into_raw(Box::new(
        created_turtles.into_iter(context.next_int.clone()),
    ))
}

#[no_mangle]
pub extern "C" fn oxitortoise_make_all_turtles_iter(
    context: &mut CanonExecutionContext,
) -> *mut TurtleIterator {
    Box::into_raw(Box::new(
        AllTurtles.into_iter(&context.workspace.world, context.next_int.clone()),
    ))
}

#[no_mangle]
pub extern "C" fn oxitortoise_next_turtle_from_iter(iter: &mut TurtleIterator) -> TurtleId {
    iter.next().unwrap_or_default()
}

/// # Safety
///
/// The caller is responsible for ensuring that the pointer points to a valid
/// iterator which can be dropped and is never used again.
#[no_mangle]
pub unsafe extern "C" fn oxitortoise_drop_turtle_iter(iter: *mut TurtleIterator) {
    drop(unsafe { Box::from_raw(iter) });
}

#[no_mangle]
pub extern "C" fn oxitortoise_make_all_patches_iter(
    context: &mut CanonExecutionContext,
) -> *mut PatchIterator {
    Box::into_raw(Box::new(
        AllPatches.into_iter(&context.workspace.world, context.next_int.clone()),
    ))
}

#[no_mangle]
pub extern "C" fn oxitortoise_next_patch_from_iter(iter: &mut PatchIterator) -> PatchId {
    iter.next().unwrap_or_default()
}

/// # Safety
///
/// The caller is responsible for ensuring that the pointer points to a valid
/// iterator which can be dropped and is never used again.
#[no_mangle]
pub unsafe extern "C" fn oxitortoise_drop_patch_iter(iter: *mut PatchIterator) {
    drop(unsafe { Box::from_raw(iter) });
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
