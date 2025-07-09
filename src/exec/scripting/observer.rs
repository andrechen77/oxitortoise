use crate::sim::{agent_schema::AgentFieldDescriptor, topology, value::Float, world::World};

/// Diffuse the specified patch variable to the neighboring eight patches. The
/// specified patch variable must be numeric.
#[no_mangle]
#[inline(never)]
pub extern "C" fn diffuse_8(
    world: &mut World,
    patch_variable: AgentFieldDescriptor,
    fraction: Float,
) {
    topology::diffuse::diffuse_8_single_variable_buffer(world, patch_variable, fraction)
}
