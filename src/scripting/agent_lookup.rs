use crate::sim::{
    patch::{Patch, PatchId},
    world::World,
};

#[no_mangle]
#[inline(never)]
pub extern "C" fn look_up_patch(world: &World, patch_id: PatchId) -> &Patch {
    &world.patches[patch_id]
}
