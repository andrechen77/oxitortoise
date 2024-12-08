use crate::{
    sim::{
        agent::AgentId,
        world::{AgentIndexIntoWorld, World},
    },
    unsafe_getter::unwrap_agent_id_mut,
};

pub fn look_up_agent<I, A>(id: AgentId, world: &mut World) -> &mut A
where
    AgentId: TryInto<I>,
    I: AgentIndexIntoWorld<Output = A>,
{
    unwrap_agent_id_mut(id, world)
}
