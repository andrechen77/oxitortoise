use std::{
    mem::offset_of,
    sync::{Arc, Mutex},
};

use crate::{
    updater::DirtyAggregator,
    util::{
        reflection::{MemRepr, Reflect, TypeInfo},
        rng::CanonRng,
    },
    workspace::Workspace,
};

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

unsafe impl<'w> Reflect for CanonExecutionContext<'w> {
    const TYPE_INFO: TypeInfo = TypeInfo::new_drop::<CanonExecutionContext<'static>>(
        "CanonExecutionContext",
        MemRepr::Compound(&[(
            offset_of!(CanonExecutionContext<'w>, workspace),
            &Workspace::TYPE_INFO,
        )]),
    );
}

unsafe impl<'w> Reflect for &'w mut CanonExecutionContext<'w> {
    const TYPE_INFO: TypeInfo =
        TypeInfo::new_mut_ref_to::<CanonExecutionContext<'static>>("&mut CanonExecutionContext");
}
