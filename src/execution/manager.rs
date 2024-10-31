use std::{collections::HashMap, rc::Rc};

use super::CommandProcedure;

#[derive(Debug)]
pub struct ProcedureManager<U> {
    commands: HashMap<Rc<str>, CommandProcedure<U>>,
    // TODO other fields
}

impl<U> Default for ProcedureManager<U> {
    fn default() -> Self {
        Self {
            commands: HashMap::default(),
        }
    }
}

impl<U> ProcedureManager<U> {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn define_command(&mut self, command: CommandProcedure<U>) {
        self.commands.insert(command.name.clone(), command);
    }

    pub fn get_command_by_name(&mut self, name: &str) -> Option<&CommandProcedure<U>> {
        self.commands.get(name)
    }
}
