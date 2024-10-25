use std::{collections::HashMap, rc::Rc};

use crate::value::Value;

#[derive(Debug, Default)]
pub struct Observer {
    variables: HashMap<Rc<str>, Value>,
    // TODO
}

impl Observer {
    pub fn get_global(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub fn create_global(&mut self, name: Rc<str>, value: Value) {
        self.variables.insert(name, value);
    }
}
