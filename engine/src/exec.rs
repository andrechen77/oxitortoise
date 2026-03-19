use std::{
    mem::offset_of,
    sync::{Arc, Mutex},
};

use crate::{mir::prelude::*, updater::DirtyAggregator, util::rng::CanonRng, workspace::Workspace};

pub mod helpers;
pub mod jit;

#[repr(C)]
pub struct ExecutionContext<'w> {
    /// The workspace in which execution is occuring.
    pub workspace: &'w mut Workspace,
    /// The output for all updates that occur during execution.
    pub next_int: Arc<Mutex<CanonRng>>,
    pub dirty_aggregator: DirtyAggregator,
}

pub type CanonExecutionContext<'w> = ExecutionContext<'w>;

impl CanonExecutionContext<'_> {
    /// Derives a `Workspace` from a `CanonExecutionContext`.
    pub fn mir_project_workspace(context: TypedPlace) -> TypedPlace {
        context
            .proj(Projection::Field { byte_offset: offset_of!(ExecutionContext, workspace) })
            .proj(Projection::Deref)
    }
}
