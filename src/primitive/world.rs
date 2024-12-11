use crate::{
    sim::{
        agent::{AgentId, AgentIndexIntoWorld},
        patch::PatchId,
        turtle::TurtleId,
        world::World,
    },
    unsafe_getter::unwrap_agent_id,
};

pub fn look_up_turtle(
    id: AgentId,
    world: &World,
) -> Option<<TurtleId as AgentIndexIntoWorld>::Output<'_>> {
    unwrap_agent_id::<TurtleId, _>(id, world)
}

pub fn look_up_patch(
    id: AgentId,
    world: &World,
) -> Option<<PatchId as AgentIndexIntoWorld>::Output<'_>> {
    unwrap_agent_id::<PatchId, _>(id, world)
}
