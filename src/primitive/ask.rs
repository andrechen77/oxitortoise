use std::mem;

use crate::sim::value::agentset::IterateAgentset;
use crate::updater::Update;

use super::{Closure, ExecutionContext};

pub fn ask<'w, A: IterateAgentset, U: Update>(
    context: &mut ExecutionContext<'w, U>,
    agentset: &mut A,
    operation: Closure<'w, U>,
) {
    let asker = mem::replace(&mut context.asker, context.executor);
    for agent in agentset.iter(context.world, context.next_int.clone()) {
        context.executor = agent.into();
        operation(context);
    }
    context.executor = mem::replace(&mut context.asker, asker);
}
