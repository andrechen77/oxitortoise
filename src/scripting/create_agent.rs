use crate::{
    sim::{
        topology::Point,
        turtle::TurtleId,
        value::{agentset::TurtleSet, Float},
    },
    updater::WriteUpdate,
};

use super::{Closure, ExecutionContext};

pub fn create_turtles_with_cmd<'w, U: WriteUpdate>(
    context: &mut ExecutionContext<'w, U>,
    count: Float,
    breed_name: &str,
    initializer_operation: Closure<'w, U>,
) {
    let mut new_turtles: Vec<TurtleId> = Vec::new();
    let count = count.to_u64_round_to_zero();
    context.world.turtles.create_turtles(
        count,
        breed_name,
        Point::ORIGIN,
        |turtle| new_turtles.push(turtle),
        &mut *context.next_int.borrow_mut(),
    );
    let mut new_turtle_set = TurtleSet::new(new_turtles);
    super::ask::ask(context, &mut new_turtle_set, initializer_operation);
}
