use std::{
    alloc::Layout,
    mem::offset_of,
    sync::{Arc, Mutex},
};

use macro_reflect::{ReflectComponents, reflect};

use crate::{
    mir::{self, MirType, MirTypeInfo},
    sim::{observer::GlobalsSchema, patch::PatchSchema, turtle::TurtleSchema, world::World},
    util::{reflection::ReflectComponents, rng::CanonRng},
};

#[derive(Debug, ReflectComponents)]
#[repr(C)]
pub struct Workspace {
    pub world: World,
    pub rng: Arc<Mutex<CanonRng>>,
}

#[reflect]
impl Reflect for Workspace {}

impl Workspace {
    /// Derives a `World` from a `Workspace`.
    pub fn mir_project_world(workspace: mir::Place) -> mir::Place {
        workspace.proj_field(offset_of!(Workspace, world))
    }

    pub fn mir_type_from_schemas(
        globals_schema: &GlobalsSchema,
        turtle_schema: &TurtleSchema,
        patch_schema: &PatchSchema,
    ) -> mir::MirType {
        let world_ty = World::mir_type_from_schemas(globals_schema, turtle_schema, patch_schema);
        mir::MirTypeInfo::with_field(Layout::new::<Self>(), offset_of!(Self, world), world_ty)
    }
}

unsafe impl ReflectComponents for &mut Workspace {
    fn mir_type() -> MirType {
        MirTypeInfo::ptr_to(Workspace::mir_type())
    }
}

#[reflect]
impl Reflect for &mut Workspace {}
