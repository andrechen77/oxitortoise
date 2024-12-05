//! Functionality for setting the built-in variables of agents.

use crate::{
    sim::{
        turtle::{Turtle, TurtleId},
        value,
    },
    execution::vm::{Execute, ExecutionContext},
    unsafe_getter::{unwrap_agent_id_mut, unwrap_option, unwrap_polyvalue},
};

#[derive(Debug)]
pub struct SetTurtleSize {}

impl<U> Execute<U> for SetTurtleSize {
    unsafe fn execute<'w>(&self, context: &mut ExecutionContext<'w, U>) {
        // get the current turtle
        let turtle: &mut Turtle =
            unwrap_agent_id_mut::<TurtleId, _>(context.executor, context.world);

        // get the size argument
        let size: value::Float = unwrap_polyvalue(unwrap_option(context.operand_stack.pop()));
        let size = size.get();

        // set the size
        turtle.set_size(size);
    }
}
