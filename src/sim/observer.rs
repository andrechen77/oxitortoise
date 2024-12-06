use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::sim::{value::PolyValue, world::World};

#[derive(Debug)]
enum GlobalVariable {
    CodeDefined(PolyValue),
    WidgetDefined(PolyValue),
}

impl GlobalVariable {
    fn get(&self) -> &PolyValue {
        match self {
            GlobalVariable::CodeDefined(value) => value,
            GlobalVariable::WidgetDefined(value) => value,
        }
    }
}

#[derive(Debug, Default)]
pub struct Observer {
    /// A back-reference to the world that includes this observer.
    world: Weak<RefCell<World>>,
    /// The global variables that are accessible to all agents.
    /// TODO use a CustomAgentVariable struct alongside a WidgetVariables struct
    variables: HashMap<Rc<str>, GlobalVariable>,
    // TODO add other fields
}

impl Observer {
    /// Sets the backreferences of this structure and all structures owned by it
    /// to point to the specified world.
    pub fn set_world(&mut self, world: Weak<RefCell<World>>) {
        self.world = world;
    }

    pub fn get_global(&self, name: &str) -> Option<&PolyValue> {
        self.variables.get(name).map(GlobalVariable::get)
    }

    pub fn create_widget_global(&mut self, name: Rc<str>, value: PolyValue) {
        self.variables
            .insert(name, GlobalVariable::WidgetDefined(value));
    }

    pub fn clear_globals(&mut self) {
        self.variables
            .retain(|_, variable| !matches!(variable, GlobalVariable::CodeDefined(_)));
    }
}