use std::rc::Rc;

use crate::{
    updater::CanonUpdater,
    util::{cell::RefCell, rng::CanonRng},
    workspace::Workspace,
};

pub mod interp;
pub mod scripting;

pub struct ExecutionContext<'w, U, R> {
    /// The workspace in which execution is occuring.
    pub workspace: &'w mut Workspace,
    /// The output for all updates that occur during execution.
    pub next_int: Rc<RefCell<R>>,
    pub updater: U,
}

pub type CanonExecutionContext<'w> = ExecutionContext<'w, CanonUpdater, CanonRng>;
