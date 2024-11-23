use std::fmt::Debug;
use std::rc::Rc;

// use super::statements::CompoundStatement;

pub struct CommandProcedure {
    pub name: Rc<str>,
    pub start: i32, // TODO consider making (start, end) its own type indicating a NetLogo source range
    pub end: i32,
    // pub action: CompoundStatement,
}

impl Debug for CommandProcedure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command")
            .field("name", &self.name)
            .field("start", &self.start)
            .field("end", &self.end)
            .finish_non_exhaustive()
    }
}
