use std::cell::RefCell;

use crate::sim::{patch::PatchId, turtle::Turtle, value, world::World};

pub fn fd_one(world: &World, turtle: &RefCell<Turtle>) {
    let mut turtle = turtle.borrow_mut();
    if let Some(new_pos) = world
        .topology
        .offset_one_by_heading(turtle.position, turtle.heading)
    {
        turtle.position = new_pos;
    }
}

pub fn can_move(world: &World, turtle: &RefCell<Turtle>, distance: value::Float) -> value::Boolean {
    let turtle = turtle.borrow_mut();
    world
        .topology
        .offset_distance_by_heading(turtle.position, turtle.heading, distance)
        .is_some()
        .into()
}

pub fn turn(turtle: &RefCell<Turtle>, angle: value::Float) {
    let mut turtle = turtle.borrow_mut();
    turtle.heading += angle;
}

pub fn patch_here<'w>(world: &'w World, turtle: &RefCell<Turtle>) -> PatchId {
    world.patch_at(turtle.borrow().position.round_to_int())
}

pub fn patch_at_angle<'w>(
    world: &'w World,
    turtle: &RefCell<Turtle>,
    angle: value::Float,
    distance: value::Float,
) -> Option<PatchId> {
    let turtle = turtle.borrow();
    let actual_heading = turtle.heading + angle;
    let new_point =
        world
            .topology
            .offset_distance_by_heading(turtle.position, actual_heading, distance)?;
    Some(world.patch_at(new_point.round_to_int()))
}
