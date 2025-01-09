use std::mem;

use crate::sim::{agent::AgentIndexIntoWorld, value::agentset::IterateAgentset};
use crate::updater::WriteUpdate;

use super::{Closure, ExecutionContext};

#[inline(never)]
pub fn ask<'w, A: IterateAgentset, U: WriteUpdate>(
    context: &mut ExecutionContext<'w, U>,
    agentset: &mut A,
    operation: Closure<'w, U>,
) {
    let asker = mem::replace(&mut context.asker, context.executor);
    for agent_id in agentset.iter(context.world, context.next_int.clone()) {
        let Some(agent) = agent_id.index_into_world(context.world) else {
            continue;
        };
        context.executor = agent.into();
        operation(context);
    }
    context.executor = mem::replace(&mut context.asker, asker);
}
