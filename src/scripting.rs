//! Not the same as a NetLogo language primitive, although closely related.
//! Items in this module represent basic functionality to manipulate model
//! state. JIT-compiled NetLogo code will call into these functions.

use crate::{
    sim::agent::Agent,
    updater::CanonUpdater,
    util::{cell::RefCell, rng::CanonRng},
    workspace::Workspace,
};

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

/// A simple function pointer type that primitives which execute other commands
/// should accept.
pub type Closure<U, R> = extern "C" fn(&mut ExecutionContext<'_, '_, U, R>);

pub type CanonExecutionContext<'w, 'rng> = ExecutionContext<'w, 'rng, CanonUpdater, CanonRng>;

pub type CanonClosure = Closure<CanonUpdater, CanonRng>;

pub mod agent_lookup;
pub mod ask;
pub mod clear;
pub mod create_agent;
pub mod math;
pub mod observer;
pub mod ticks;
pub mod topology;
pub mod turtle;

pub use agent_lookup::*;
pub use ask::*;
pub use clear::*;
pub use create_agent::*;
pub use math::*;
pub use observer::*;
pub use ticks::*;
pub use topology::*;
pub use turtle::*;
