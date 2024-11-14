use std::{cell::RefCell, rc::Rc};

use crate::{patch::PatchId, turtle::Turtle, world::World};

#[derive(Debug, Clone)]
pub enum Agent {
    Observer(Rc<RefCell<World>>),
    Turtle(Rc<RefCell<Turtle>>),
    Patch(Rc<RefCell<World>>, PatchId),
    Link(/* TODO */),
}

impl Agent {
    pub fn get_world(&self) -> Rc<RefCell<World>> {
        match self {
            Agent::Observer(world) => world.clone(),
            Agent::Turtle(turtle) => turtle.borrow().get_world(),
            Agent::Patch(world, _) => world.clone(),
            Agent::Link(/* TODO */) => unimplemented!(),
        }
    }
}
