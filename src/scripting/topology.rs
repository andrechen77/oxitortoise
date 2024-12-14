use crate::sim::{
    agent::AgentPosition,
    topology::{self, Point},
    value,
    world::World,
};

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

pub fn min_pxcor(world: &World) -> value::Float {
    world.topology.min_pxcor().into()
}

pub fn max_pxcor(world: &World) -> value::Float {
    world.topology.max_pxcor().into()
}

pub fn min_pycor(world: &World) -> value::Float {
    world.topology.min_pycor().into()
}

pub fn max_pycor(world: &World) -> value::Float {
    world.topology.max_pycor().into()
}
