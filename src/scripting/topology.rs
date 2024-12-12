use crate::sim::{
    agent::AgentPosition,
    topology::{self, Point},
    value,
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
