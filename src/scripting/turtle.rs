use crate::sim::{patch::PatchId, turtle::Turtle, value, world::World};

pub fn fd_one(world: &World, turtle: &Turtle) {
    let mut turtle_data = turtle.data.borrow_mut();
    if let Some(new_pos) = world
        .topology
        .offset_one_by_heading(turtle_data.position, turtle_data.heading)
    {
        turtle_data.position = new_pos;
    }
}

pub fn can_move(world: &World, turtle: &Turtle, distance: value::Float) -> value::Boolean {
    let turtle_data = turtle.data.borrow();
    world
        .topology
        .offset_distance_by_heading(turtle_data.position, turtle_data.heading, distance)
        .is_some()
        .into()
}

pub fn turn(turtle: &Turtle, angle: value::Float) {
    turtle.data.borrow_mut().heading += angle;
}

pub fn patch_here(world: &World, turtle: &Turtle) -> PatchId {
    world.patch_at(turtle.data.borrow().position.round_to_int())
}

pub fn patch_at_angle(
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
    Some(world.patch_at(new_point.round_to_int()))
}
