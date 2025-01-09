use crate::sim::{
    agent::AgentPosition,
    topology::{self, Point},
    value,
    world::World,
};

#[inline(never)]
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

#[inline(never)]
pub fn min_pxcor(world: &World) -> value::Float {
    world.topology.min_pxcor().into()
}

#[inline(never)]
pub fn max_pxcor(world: &World) -> value::Float {
    world.topology.max_pxcor().into()
}

#[inline(never)]
pub fn min_pycor(world: &World) -> value::Float {
    world.topology.min_pycor().into()
}

#[inline(never)]
pub fn max_pycor(world: &World) -> value::Float {
    world.topology.max_pycor().into()
}
