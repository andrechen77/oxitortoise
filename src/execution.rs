// TODO document the difference between instructions and

use std::{cell::RefCell, rc::Rc};

use crate::{agent::AgentId, value::{self, PolyValue}, world::World};

mod manager;
mod procedure;
// mod statements;
mod monolithic;
mod polylithic;
mod pieces;

pub use manager::ProcedureManager;

/// Holds
pub struct ExecutionContext<'w, U, R> {
    /// The world in which the execution is occurring.
    world: &'w mut World,
    /// The agent that is executing the current command. The `self` reporter in
    /// NetLogo returns this value if it is not the observer.
    executor: AgentId,
    /// The agent that asked the executor to execute the current command. The
    /// `myself` reporter in NetLogo returns this value if it is not the
    /// observer.
    asker: AgentId,
    /// A stack of values used by the non-opaque commands to remember values
    /// during a thread of execution.
    value_stack: Vec<PolyValue>,
    /// `None` if not inside the catch block of a `carefully`; this represents
    /// the message of the error currently being handled.
    error: Option<value::String>,
    /// The output for all updates that occur during execution.
    updater: U,
    next_int: Rc<RefCell<R>>,
    // TODO rngs, etc.
}
