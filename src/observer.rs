use std::{cell::RefCell, collections::HashMap, rc::{Rc, Weak}};

use crate::{value::Value, world::World};

#[derive(Debug, Default)]
pub struct Observer {
    /// A back-reference to the world that includes this observer.
    world: Weak<RefCell<World>>,
    /// The global variables that are accessible to all agents.
    variables: HashMap<Rc<str>, Value>,
    // TODO
}

impl Observer {
    /// Sets the backreferences of this structure and all structures owned by it
    /// to point to the specified world.
    pub fn set_world(&mut self, world: Weak<RefCell<World>>) {
        self.world = world;
    }

    pub fn get_global(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub fn create_global(&mut self, name: Rc<str>, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn clear_globals(&mut self) {
        self.variables.clear();
    }
}
