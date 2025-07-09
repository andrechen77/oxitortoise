use crate::{
    exec::CanonExecutionContext,
    sim::{
        topology::Point,
        turtle::{BreedId, TurtleId},
        value::{agentset::TurtleSet, Float},
    },
};

#[no_mangle]
#[inline(never)]
pub extern "C" fn create_turtles(
    context: &mut CanonExecutionContext,
    breed_id: BreedId,
    count: u64,
    spawn_point: Point,
) -> TurtleSet {
    context.workspace.world.turtles.create_turtles(
        breed_id,
        count,
        spawn_point,
        &mut *context.next_int.borrow_mut(),
    )
}

// use super::CanonClosure;

// #[no_mangle]
// #[inline(never)]
// pub extern "C" fn create_turtles_with_cmd(
//     context: &mut CanonExecutionContext,
//     count: Float,
//     breed_id: BreedId,
//     initializer_operation: CanonClosure,
// ) {
//     let mut new_turtles: Vec<TurtleId> = Vec::new();
//     let count = count.to_u64_round_to_zero();
//     context.workspace.world.turtles.create_turtles(
//         count,
//         breed_id,
//         Point::ORIGIN,
//         |turtle| new_turtles.push(turtle),
//         &mut *context.next_int.borrow_mut(),
//         &context.workspace.world.shapes,
//     );
//     let mut new_turtle_set = TurtleSet::new(new_turtles);
//     super::ask::ask(context, &mut new_turtle_set, initializer_operation);
// }
