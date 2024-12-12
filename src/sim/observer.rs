use std::{
    collections::HashMap,
    rc::Rc,
};

use crate::sim::value::PolyValue;

// TOOD change observer to use the same variable system as the agents

#[derive(Debug)]
enum GlobalVariable {
    #[allow(dead_code)]
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
    /// The global variables that are accessible to all agents.
    /// TODO use a CustomAgentVariable struct alongside a WidgetVariables struct
    variables: HashMap<Rc<str>, GlobalVariable>,
    // TODO add other fields
}

impl Observer {
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
