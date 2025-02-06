use std::mem;

use crate::sim::{
    agent::AgentIndexIntoWorld,
    value::agentset::{AllPatches, AllTurtles, IterateAgentset},
};

use super::{CanonClosure, CanonExecutionContext};

#[no_mangle]
#[inline(never)]
pub extern "C" fn ask_all_turtles(context: &mut CanonExecutionContext, operation: CanonClosure) {
    ask(context, &mut AllTurtles, operation);
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn ask_all_patches(context: &mut CanonExecutionContext, operation: CanonClosure) {
    ask(context, &mut AllPatches, operation);
}

pub fn ask<A: IterateAgentset>(
    context: &mut CanonExecutionContext,
    agentset: &mut A,
    operation: CanonClosure,
) {
    let asker = mem::replace(&mut context.asker, context.executor);
    for agent_id in agentset.iter(&context.workspace.world, context.next_int) {
        let Some(agent) = agent_id.index_into_world(&context.workspace.world) else {
            continue;
        };
        context.executor = agent.into();
        operation(context);
    }
    context.executor = mem::replace(&mut context.asker, asker);
}
