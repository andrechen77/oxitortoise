//! The building blocks of a NetLogo program.

use crate::updater::Update;

use super::{functionality, Execute, ExecutionContext};

pub use functionality::clear_all::ClearAll;
pub use functionality::create_turtles::ContinueCrtAsk;
pub use functionality::create_turtles::CreateTurtlesWithCommands;

#[derive(Debug)]
pub enum Instruction {
    ClearAll(ClearAll),
    CreateTurtles(CreateTurtlesWithCommands),
}

impl<U: Update> Execute<U> for Instruction {
    unsafe fn execute<'w>(&self, context: &mut ExecutionContext<'w, U>) {
        match self {
            Instruction::ClearAll(instruction) => instruction.execute(context),
            Instruction::CreateTurtles(instruction) => instruction.execute(context),
        }
    }
}
