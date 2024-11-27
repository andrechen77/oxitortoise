use std::{collections::HashMap, rc::Rc};

use crate::execution::procedure::Procedure;

#[derive(Debug)]
pub struct ProcedureManager {
    procedures: HashMap<Rc<str>, Procedure>,
    // TODO other fields
}

impl Default for ProcedureManager {
    fn default() -> Self {
        Self {
            procedures: HashMap::default(),
        }
    }
}

impl ProcedureManager {
    pub fn new() -> Self {
        Self {
            procedures: HashMap::new(),
        }
    }

    pub fn define_procedure(&mut self, procedure: Procedure) {
        self.procedures.insert(procedure.name().clone(), procedure);
    }

    pub fn get_procedure(&mut self, name: &str) -> Option<&Procedure> {
        self.procedures.get(name)
    }
}
