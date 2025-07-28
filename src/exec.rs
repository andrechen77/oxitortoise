use std::{mem::offset_of, rc::Rc};

use crate::{
    updater::CanonUpdater,
    util::{cell::RefCell, rng::CanonRng},
    workspace::Workspace,
};

pub mod dynamic_link;
pub mod interp;
pub mod jit;

#[no_mangle]
static OFFSET_CONTEXT_TO_WORKSPACE: usize = offset_of!(CanonExecutionContext, workspace);

#[no_mangle]
static OFFSET_CONTEXT_TO_UPDATER: usize = offset_of!(CanonExecutionContext, updater);

#[repr(C)]
pub struct ExecutionContext<'w, U, R> {
    /// The workspace in which execution is occuring.
    pub workspace: &'w mut Workspace,
    /// The output for all updates that occur during execution.
    pub next_int: Rc<RefCell<R>>,
    pub updater: U,
}

pub type CanonExecutionContext<'w> = ExecutionContext<'w, CanonUpdater, CanonRng>;
