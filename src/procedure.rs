use std::{collections::HashMap, fmt::Debug, rc::Rc};

use crate::{agent_id::AgentId, updater::Update, world::World};

pub struct Command {
    pub name: Rc<str>,
    pub start: i32, // TODO consider making (start, end) its own type indicating a NetLogo source range
    pub end: i32,
    pub action: AnonCommand,
}

/// A command in the NetLogo execution model. When run, a command modifies the
/// given world from the perspective of the given agent, and outputs all updates
/// to the given updater.
pub type AnonCommand = Box<dyn Fn(&mut World, AgentId, &mut dyn Update)>;

impl Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command")
            .field("name", &self.name)
            .field("start", &self.start)
            .field("end", &self.end)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct ProcedureManager {
    commands: HashMap<Rc<str>, Command>,
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

    pub fn define_command(&mut self, command: Command) {
        self.commands.insert(command.name.clone(), command);
    }

    pub fn get_command_by_name(&mut self, name: &str) -> Option<&Command> {
        self.commands.get(name)
    }
}
