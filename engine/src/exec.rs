use std::{mem::offset_of, rc::Rc};

use crate::{
    updater::DirtyAggregator,
    util::{cell::RefCell, rng::CanonRng},
    workspace::Workspace,
};

pub mod helpers;
pub mod jit;

#[no_mangle]
static OFFSET_CONTEXT_TO_WORKSPACE: usize = offset_of!(CanonExecutionContext, workspace);

#[no_mangle]
static OFFSET_CONTEXT_TO_DIRTY_AGGREGATOR: usize =
    offset_of!(CanonExecutionContext, dirty_aggregator);

#[repr(C)]
pub struct ExecutionContext<'w> {
    /// The workspace in which execution is occuring.
    pub workspace: &'w mut Workspace,
    /// The output for all updates that occur during execution.
    pub next_int: Rc<RefCell<CanonRng>>,
    pub dirty_aggregator: DirtyAggregator,
}

pub type CanonExecutionContext<'w> = ExecutionContext<'w>;
