use std::cell::RefCell;

use crate::sim::{turtle::Turtle, world::World};

pub fn fd_one(world: &World, turtle: &RefCell<Turtle>) {
    let mut turtle = turtle.borrow_mut();
    if let Some(new_pos) = world
        .topology
        .offset_by_heading(turtle.position, turtle.heading)
    {
        turtle.position = new_pos;
    }
}
