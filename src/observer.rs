use std::collections::HashMap;

use crate::value::Value;

#[derive(Debug, Default)]
pub struct Observer {
    variables: HashMap<String, Value>,
    // TODO
}

impl Observer {
    pub fn get_global(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }
}
