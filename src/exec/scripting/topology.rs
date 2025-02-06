use crate::sim::{
    agent::AgentPosition,
    patch::Patch,
    topology::{self, Point},
    value,
    world::World,
};

#[no_mangle]
#[inline(never)]
pub extern "C" fn distancexy_euclidean_patch(
    agent: &Patch,
    x: value::Float,
    y: value::Float,
) -> value::Float {
    distancexy_euclidean(agent, x, y)
}

pub fn distancexy_euclidean<A: AgentPosition>(
    agent: &A,
    x: value::Float,
    y: value::Float,
) -> value::Float {
    topology::euclidean_distance_unwrapped(
        agent.position(),
        Point {
            x: x.get(),
            y: y.get(),
        },
    )
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn min_pxcor(world: &World) -> value::Float {
    world.topology.min_pxcor().into()
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn max_pxcor(world: &World) -> value::Float {
    world.topology.max_pxcor().into()
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn min_pycor(world: &World) -> value::Float {
    world.topology.min_pycor().into()
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn max_pycor(world: &World) -> value::Float {
    world.topology.max_pycor().into()
}
