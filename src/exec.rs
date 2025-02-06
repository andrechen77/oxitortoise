use crate::{
    sim::agent::Agent,
    updater::CanonUpdater,
    util::{cell::RefCell, rng::CanonRng},
    workspace::Workspace,
};

pub mod interp;
pub mod scripting;

pub struct ExecutionContext<'w, 'rng, U, R> {
    /// The workspace in which execution is occuring.
    pub workspace: &'w Workspace,
    /// The agent that is executing the current command. The `self` reporter in
    /// NetLogo returns this value if it is not the observer.
    pub executor: Agent<'w>,
    /// The agent that asked the executor to execute the current command. The
    /// `myself` reporter in NetLogo returns this value if it is not the
    /// observer.
    pub asker: Agent<'w>,
    /// The output for all updates that occur during execution.
    pub next_int: &'rng RefCell<R>,
    pub updater: U,
}

pub type CanonExecutionContext<'w, 'rng> = ExecutionContext<'w, 'rng, CanonUpdater, CanonRng>;
