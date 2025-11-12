use std::{cell::RefCell, rc::Rc};

use crate::{updater::DirtyAggregator, util::rng::CanonRng, workspace::Workspace};

pub mod helpers;
pub mod jit;

#[repr(C)]
pub struct ExecutionContext<'w> {
    /// The workspace in which execution is occuring.
    pub workspace: &'w mut Workspace,
    /// The output for all updates that occur during execution.
    pub next_int: Rc<RefCell<CanonRng>>,
    pub dirty_aggregator: DirtyAggregator,
}

pub type CanonExecutionContext<'w> = ExecutionContext<'w>;
