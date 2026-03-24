//! Helper functions for operations that involve the execution context.

use crate::{
    sim::{
        patch::PatchId,
        topology::Point,
        turtle::{BreedId, TurtleId},
        value::{
            NlFloat,
            agentset::{shuffled_patches, shuffled_turtles},
        },
    },
    util::rng::Rng,
    workspace::Workspace,
};

pub fn create_turtles<R: Rng>(
    workspace: &mut Workspace,
    rng: &mut R,
    breed: BreedId,
    count: NlFloat,
    position: Point,
    mut birth_command: impl FnMut(&mut Workspace, &mut R, TurtleId),
) {
    let new_turtles =
        workspace.world.turtles.create_turtles(breed, count.to_u64_round_to_zero(), position, rng);

    let mut iter = new_turtles.into_shuffler();
    while let Some(turtle) = iter.next(rng) {
        birth_command(workspace, rng, turtle);
    }
}

pub fn for_all_turtles<R: Rng>(
    workspace: &mut Workspace,
    rng: &mut R,
    mut block: impl FnMut(&mut Workspace, &mut R, TurtleId),
) {
    let mut iter = shuffled_turtles(&workspace.world);
    while let Some(turtle) = iter.next(rng) {
        block(workspace, rng, turtle);
    }
}

pub fn for_all_patches<R: Rng>(
    workspace: &mut Workspace,
    rng: &mut R,
    mut block: impl FnMut(&mut Workspace, &mut R, PatchId),
) {
    let mut iter = shuffled_patches(&workspace.world);
    while let Some(patch) = iter.next(rng) {
        block(workspace, rng, patch);
    }
}
