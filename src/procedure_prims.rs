use std::{collections::HashMap, fmt::Debug, rc::Rc};

use crate::workspace::Workspace;

pub struct Command {
    pub name: Rc<str>,
    pub start: i32, // TODO consider making (start, end) its own type indicating a NetLogo source range
    pub end: i32,
    pub action: Box<dyn FnMut(&mut Workspace)>,
}

impl Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command")
            .field("name", &self.name)
            .field("start", &self.start)
            .field("end", &self.end)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Default)]
pub struct ProcedurePrims {
    // eval_prims: EvalPrims,
    // plot_manager: PlotManager,
    commands: HashMap<String, Command>,
    // reporters: HashMap<String, Reporter>,
    // stack: ProcedureStack,
}

impl ProcedurePrims {
    pub fn new() -> Self {
        ProcedurePrims::default()
    }

    pub fn define_command(&mut self, command: Command) {
        self.commands.insert((*command.name).to_owned(), command);
    }
}
