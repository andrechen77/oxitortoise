use std::{mem, ops::DerefMut as _, rc::Rc};

use flagset::FlagSet;

use crate::{
    agent::AgentId, execution::{
        monolithic::{ExecuteMonolith, StatementMonolith},
        ExecutionContext,
    }, rng::NextInt, shuffle_iterator::ShuffleIterator, topology::Point, updater::Update, value::{self, PolyValue}
};

use super::*;

pub struct CreateTurtlesMonolith {
    /// The statement to evaluate to determine how many turtles to create.
    count: StatementMonolith,
    /// The name of the breed of turtles to create.
    breed_name: Rc<str>,
    /// The statements to run for each turtle after they are created.
    commands: Option<CompoundStatementMonolith>,
}

impl ExecuteMonolith for CreateTurtlesMonolith {
    fn execute_monolith<'w, U, R>(
        &self,
        context: &mut ExecutionContext<'w, U, R>,
    ) -> PolyValue
    where
        U: Update,
        R: NextInt,
    {
        // TODO must be executed by the observer only

        // evaluate the count argument
        let Some(count) = self.count.execute_monolith(context).into::<value::Float>() else {
            panic!("input to `create-turtles` must return a number");
        };
        let count = count.to_u64_round_to_zero();

        // create the turtles
        let mut new_turtles = Vec::with_capacity(count as usize);
        context.world.turtles.create_turtles(
            count,
            &self.breed_name,
            Point::ORIGIN,
            |turtle| new_turtles.push(turtle),
            context.next_int.borrow_mut().deref_mut(),
        );

        // run the commands on the turtles
        if let Some(commands) = &self.commands {
            // set up the proper asker/executor for the commands
            let my_asker = mem::replace(&mut context.asker, context.executor);

            for turtle in ShuffleIterator::new(&mut new_turtles, Rc::clone(&context.next_int)) {
                context.executor = AgentId::Turtle(*turtle);

                commands.execute_monolith(context);

                context.updater.update_turtle(
                    context.world.turtles.get_by_index(*turtle).expect("turtle was just created"),
                    FlagSet::default(),
                );
            }

            // restore the asker/executor
            context.executor = mem::replace(&mut context.asker, my_asker);
        } else {
            for turtle in new_turtles {
                context.updater.update_turtle(
                    context.world.turtles.get_by_index(turtle).expect("turtle was just created"),
                    FlagSet::default(),
                );
            }
        }

        PolyValue::ERASED
    }
}
