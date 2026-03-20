use std::{
    alloc::Layout,
    mem::offset_of,
    sync::{Arc, Mutex},
};

use crate::{
    mir::prelude::*,
    sim::{observer::GlobalsSchema, patch::PatchSchema, turtle::TurtleSchema},
    updater::DirtyAggregator,
    util::rng::CanonRng,
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

impl CanonExecutionContext<'_> {
    /// Derives a `Workspace` from a `CanonExecutionContext`.
    pub fn mir_project_workspace(context: TypedPlace) -> TypedPlace {
        context
            .proj(Projection::Field { byte_offset: offset_of!(ExecutionContext, workspace) })
            .proj(Projection::Deref)
    }

    pub fn mir_type_from_schemas(
        globals_schema: &GlobalsSchema,
        turtle_schema: &TurtleSchema,
        patch_schema: &PatchSchema,
    ) -> MirType {
        let workspace_ty =
            Workspace::mir_type_from_schemas(globals_schema, turtle_schema, patch_schema);
        MirTypeInfo::with_field(
            Layout::new::<Self>(),
            offset_of!(Self, workspace),
            MirTypeInfo::ptr_to(workspace_ty),
        )
    }
}
