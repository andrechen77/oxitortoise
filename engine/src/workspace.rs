use std::{
    mem::offset_of,
    sync::{Arc, Mutex},
};

use crate::{
    sim::world::World,
    util::{
        reflection::{MemRepr, Reflect, TypeInfo},
        rng::CanonRng,
    },
};

#[derive(Debug)]
#[repr(C)]
pub struct Workspace {
    pub world: World,
    pub rng: Arc<Mutex<CanonRng>>,
}

unsafe impl Reflect for Workspace {
    const TYPE_INFO: TypeInfo = TypeInfo::new_drop::<Workspace>(
        "Workspace",
        MemRepr::Compound(&[(offset_of!(Workspace, world), &World::TYPE_INFO)]),
    );
}

unsafe impl<'a> Reflect for &'a mut Workspace {
    const TYPE_INFO: TypeInfo = TypeInfo::new_mut_ref_to::<Workspace>("&mut Workspace");
}
