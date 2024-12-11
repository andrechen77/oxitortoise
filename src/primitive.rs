//! Not the same as a NetLogo language primitive, although closely related.
//! Items in this module represent basic functionality to manipulate model
//! state. JIT-compiled NetLogo code will call into these functions.

use std::{cell::RefCell, rc::Rc};

use crate::{
    sim::{agent::AgentId, world::World},
    util::rng::CanonRng,
};

pub struct ExecutionContext<'w, U> {
    /// The world in which the execution is occurring.
    pub world: &'w World,
    /// The agent that is executing the current command. The `self` reporter in
    /// NetLogo returns this value if it is not the observer.
    pub executor: AgentId,
    /// The agent that asked the executor to execute the current command. The
    /// `myself` reporter in NetLogo returns this value if it is not the
    /// observer.
    pub asker: AgentId,
    /// The output for all updates that occur during execution.
    pub updater: U,
    pub next_int: Rc<RefCell<CanonRng>>,
}

/// A simple function pointer type that primitives which execute other commands
/// should accept.
pub type Closure<'w, U> = fn(&mut ExecutionContext<'w, U>);

pub mod ask;
pub mod clear;
pub mod create_agent;
pub mod topology;
pub mod world;
