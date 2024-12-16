use std::cell::RefCell;

use crate::sim::{turtle::Turtle, value, world::World};

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
