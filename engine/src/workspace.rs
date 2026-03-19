use std::{
    mem::offset_of,
    sync::{Arc, Mutex},
};

use crate::{mir::prelude::*, sim::world::World, util::rng::CanonRng};

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
}
