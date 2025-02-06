//! Not the same as a NetLogo language primitive, although closely related.
//! Items in this module represent basic functionality to manipulate model
//! state. JIT-compiled NetLogo code will call into these functions.

use crate::{updater::CanonUpdater, util::rng::CanonRng};

use super::ExecutionContext;

/// A simple function pointer type that primitives which execute other commands
/// should accept.
pub type Closure<U, R> = extern "C" fn(&mut ExecutionContext<'_, '_, U, R>);

pub type CanonClosure = Closure<CanonUpdater, CanonRng>;

mod agent_lookup;
mod ask;
mod clear;
mod create_agent;
mod math;
mod observer;
mod ticks;
mod topology;
mod turtle;

pub mod prelude {
    use super::*;

    pub use super::CanonClosure;
    pub use agent_lookup::*;
    pub use ask::*;
    pub use clear::*;
    pub use create_agent::*;
    pub use math::*;
    pub use observer::*;
    pub use ticks::*;
    pub use topology::*;
    pub use turtle::*;
}
