use crate::sim::{agent_variables::VarIndex, topology, value::Float, world::World};

// TODO should work with some built-in variables as well; i.e. any variabledescriptor.
// this todo should naturally be done when the new system for variabledescriptors
// and agent variables with variable-length structs is implemented. when this is
// done, keep in mind that pcolor should be able to diffuse even though it's
// not a Float, it's a Color.
/// Diffuse the specified patch variable to the neighboring eight patches. The
/// specified patch variable must be numeric.
#[inline(never)]
pub fn diffuse_8(world: &World, patch_variable: VarIndex, fraction: Float) {
    topology::diffuse::diffuse_8(world, patch_variable, fraction);
}
