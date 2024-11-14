use std::{collections::HashMap, rc::Rc};

use crate::execution::procedure::CommandProcedure;

#[derive(Debug)]
pub struct ProcedureManager {
    commands: HashMap<Rc<str>, CommandProcedure>,
    // TODO other fields
}

impl Default for ProcedureManager {
    fn default() -> Self {
        Self {
            commands: HashMap::default(),
        }
    }
}

impl ProcedureManager {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn define_command(&mut self, command: CommandProcedure) {
        self.commands.insert(command.name.clone(), command);
    }

    pub fn get_command_by_name(&mut self, name: &str) -> Option<&CommandProcedure> {
        self.commands.get(name)
    }
}
