use crate::{
    agent::Agent, value,
};

mod manager;
mod procedure;
mod statements;

pub use manager::ProcedureManager;

/// Holds
pub struct ExecutionContext<'r, U, R> {
    /// The agent that is executing the current command. The `self` reporter in
    /// NetLogo returns this value if it is not the observer.
    executor: Agent,
    /// The agent that asked the executor to execute the current command. The
    /// `myself` reporter in NetLogo returns this value if it is not the
    /// observer.
    asker: Agent,
    /// The output for all updates that occur during execution.
    updater: U,
    /// `None` if not inside the catch block of a `carefully`; this represents
    /// the message of the error currently being handled.
    error: Option<value::String>,
    next_int: &'r mut R,
    // TODO rngs, etc.
}
