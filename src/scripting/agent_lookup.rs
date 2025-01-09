use crate::sim::{
    patch::{Patch, PatchId},
    world::World,
};

#[inline(never)]
pub fn look_up_patch(world: &World, patch_id: PatchId) -> &Patch {
    &world.patches[patch_id]
}
