use crate::sim::{
    agent::{AgentId, AgentIndexIntoWorld},
    value::{ContainedInValue, PolyValue},
    world::World,
};

pub(crate) fn unwrap_option<T>(option: Option<T>) -> T {
    #[cfg(debug_assertions)]
    return option.unwrap();

    #[cfg(not(debug_assertions))]
    return option.unwrap_unchecked();
}

pub(crate) fn unwrap_polyvalue<T: ContainedInValue>(polyvalue: PolyValue) -> T {
    #[cfg(debug_assertions)]
    return polyvalue.into().unwrap();

    #[cfg(not(debug_assertions))]
    return polyvalue.into().unwrap_unchecked(); // TODO or use into_unchecked
}

/// Uses an agent ID to get the corresponding agent from the world.
pub(crate) fn unwrap_agent_id<'w, I, A>(id: AgentId, world: &'w World) -> Option<A>
where
    AgentId: TryInto<I>,
    I: AgentIndexIntoWorld<Output<'w> = A>,
{
    #[cfg(debug_assertions)]
    {
        let id: I = id
            .try_into()
            .ok()
            .expect("agent should be of the correct type");
        return id.index_into_world(world);
    }

    #[cfg(not(debug_assertions))]
    {
        let id: I = id.try_into().unwrap_unchecked();
        return id.index_into_world(world);
    }
}

pub(crate) fn convert<T, U>(value: T) -> U
where
    T: TryInto<U>,
{
    #[cfg(debug_assertions)]
    return value
        .try_into()
        .ok()
        .expect("value should be of the correct type");

    #[cfg(not(debug_assertions))]
    return value.try_into().unwrap_unchecked();
}
