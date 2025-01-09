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
pub extern "C" fn can_move(world: &World, turtle: &Turtle, distance: value::Float) -> value::Boolean {
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

#[no_mangle]
#[inline(never)]
pub extern "C" fn patch_at_angle(
    world: &World,
    turtle: &Turtle,
    angle: value::Float,
    distance: value::Float,
) -> Option<PatchId> {
    let turtle_data = turtle.data.borrow();
    let actual_heading = turtle_data.heading + angle;
    let new_point = world.topology.offset_distance_by_heading(
        turtle_data.position,
        actual_heading,
        distance,
    )?;
    Some(world.topology.patch_at(new_point.round_to_int()))
}
