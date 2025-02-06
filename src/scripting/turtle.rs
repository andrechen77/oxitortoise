use crate::sim::{patch::PatchId, turtle::Turtle, value, world::World};

#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_one(world: &World, turtle: &Turtle) {
    let mut turtle_data = turtle.data.borrow_mut();
    if let Some(new_pos) = world
        .topology
        .offset_one_by_heading(turtle_data.position, turtle_data.heading)
    {
        turtle_data.position = new_pos;
    }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn can_move(
    world: &World,
    turtle: &Turtle,
    distance: value::Float,
) -> value::Boolean {
    let turtle_data = turtle.data.borrow();
    world
        .topology
        .offset_distance_by_heading(turtle_data.position, turtle_data.heading, distance)
        .is_some()
        .into()
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn turn(turtle: &Turtle, angle: value::Float) {
    turtle.data.borrow_mut().heading += angle;
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn patch_here(world: &World, turtle: &Turtle) -> PatchId {
    world
        .topology
        .patch_at(turtle.data.borrow().position.round_to_int())
}

/// # Return type
///
/// This function returns what is conceptually an `Option<PatchId>`. To make it
/// compatible with the C ABI, this has been changed to actually return a
/// `usize`. A value of `usize::MAX` is used to represent `None`, and any other
/// value transparently represents some `PatchId`.
// TODO find a way to better represent an Option, ideally with richer ABI if one
// will ever exist in Rust.
#[no_mangle]
#[inline(never)]
pub extern "C" fn patch_at_angle(
    world: &World,
    turtle: &Turtle,
    angle: value::Float,
    distance: value::Float,
) -> usize {
    let turtle_data = turtle.data.borrow();
    let actual_heading = turtle_data.heading + angle;
    let Some(new_point) =
        world
            .topology
            .offset_distance_by_heading(turtle_data.position, actual_heading, distance)
    else {
        return usize::MAX;
    };
    world.topology.patch_at(new_point.round_to_int()).0
}
