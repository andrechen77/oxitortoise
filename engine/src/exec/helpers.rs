//! Helper functions for operations that involve the execution context.

use crate::{
    exec::ExecutionContext,
    sim::{
        patch::PatchId,
        topology::Point,
        turtle::{BreedId, TurtleId},
        value::agentset::{AllPatches, AllTurtles},
    },
};

pub fn create_turtles(
    context: &mut ExecutionContext,
    breed: BreedId,
    count: f64,
    position: Point,
    mut birth_command: impl FnMut(&mut ExecutionContext, TurtleId),
) {
    let new_turtles = context.workspace.world.turtles.create_turtles(
        breed,
        count as u64, // TODO look into rounding behavior
        position,
        &mut context.next_int,
    );

    // foreign code may rely on the fact that the dirty aggregator has enough
    // space for all turtles.
    context
        .dirty_aggregator
        .reserve_turtles(context.workspace.world.turtles.num_turtles() as usize);

    for turtle in new_turtles.into_iter(context.next_int.clone()) {
        birth_command(context, turtle);
    }
}

pub fn for_all_turtles(
    context: &mut ExecutionContext,
    mut block: impl FnMut(&mut ExecutionContext, TurtleId),
) {
    for turtle in AllTurtles.into_iter(&context.workspace.world, context.next_int.clone()) {
        block(context, turtle);
    }
}

pub fn for_all_patches(
    context: &mut ExecutionContext,
    mut block: impl FnMut(&mut ExecutionContext, PatchId),
) {
    for patch in AllPatches.into_iter(&context.workspace.world, context.next_int.clone()) {
        block(context, patch);
    }
}
