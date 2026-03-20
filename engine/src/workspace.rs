use std::{
    alloc::Layout,
    mem::offset_of,
    sync::{Arc, Mutex},
};

use crate::{
    mir::prelude::*,
    sim::{observer::GlobalsSchema, patch::PatchSchema, turtle::TurtleSchema, world::World},
    util::rng::CanonRng,
};

#[derive(Debug)]
#[repr(C)]
pub struct Workspace {
    pub world: World,
    pub rng: Arc<Mutex<CanonRng>>,
}

impl Workspace {
    /// Derives a `World` from a `Workspace`.
    pub fn mir_project_world(workspace: TypedPlace) -> TypedPlace {
        workspace.proj(Projection::Field { byte_offset: offset_of!(Workspace, world) })
    }

    pub fn mir_type_from_schemas(
        globals_schema: &GlobalsSchema,
        turtle_schema: &TurtleSchema,
        patch_schema: &PatchSchema,
    ) -> MirType {
        let world_ty = World::mir_type_from_schemas(globals_schema, turtle_schema, patch_schema);
        MirTypeInfo::with_field(Layout::new::<Self>(), offset_of!(Self, world), world_ty)
    }
}
