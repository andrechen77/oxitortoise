use std::{cell::RefCell, rc::Rc};

use crate::{turtle::Turtle, world::World};

#[derive(Debug, Clone)]
pub enum Agent {
    Observer(Rc<RefCell<World>>),
    Turtle(Rc<RefCell<Turtle>>),
    Patch(/* TODO */),
    Link(/* TODO */),
}

impl Agent {
    pub fn get_world(&self) -> Rc<RefCell<World>> {
        match self {
            Agent::Observer(world) => world.clone(),
            Agent::Turtle(turtle) => turtle.borrow().get_world(),
            Agent::Patch(/* TODO */) => unimplemented!(),
            Agent::Link(/* TODO */) => unimplemented!(),
        }
    }
}
