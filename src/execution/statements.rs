use std::{mem, rc::Rc};

use flagset::FlagSet;

use crate::{
    agent::Agent,
    rng::NextInt,
    updater::Update,
    value::{self, Value},
};

use super::ExecutionContext;

pub enum StatementOutcome {
    /// The statement executed successfully and returned. If the statement was
    /// a reporter, then the value is returned.
    Return(Option<Value>),
    /// While executing the statement, an early return was encountered. This
    /// aborts execution of all statements higher up in the call stack up to the
    /// next procedure boundary. If we are currently a reporter, then the value
    /// is returned.
    EarlyReturn(Option<Value>),
    /// While executing the statement, an error occurred. This aborts execution
    /// of all statements higher up in the call stack up to the next `carefully`
    /// statement.
    Error(value::String),
}

/// Represents an executable statement that the engine can execute. A statement
/// may or may not return a value and may or may not correspond exactly to the
/// NetLogo language's concept of a command/reporter.
pub trait Statement {
    /// Executes the statement with the specified execution context and returns
    /// the outcome. All updates that the statement would like to propagate
    /// must go into the updater of the given execution context. As as general
    /// rule of good practice, a statement should be able to assume that all
    /// the state in its execution context (which uses RefCell heavily) is
    /// currently unborrowed, so a statement should make sure that it stops
    /// borrowing that state when it calls other statements.
    fn execute<'r, U, R>(&self, context: &mut ExecutionContext<'r, U, R>) -> StatementOutcome
    where
        U: Update,
        R: NextInt;
}

/// A enum covering all possible kinds of statements. TODO.
// Because the size of an enum is the size of its largest variant, we should
// aim to keep all variant small; larger variants should be boxed.
pub enum StatementKind {
    CompoundStatement(CompoundStatement),
}

// essentially manually doing dynamic dispatch for `StatementKind`
impl Statement for StatementKind {
    fn execute<'r, U, R>(&self, context: &mut ExecutionContext<'r, U, R>) -> StatementOutcome
    where
        U: Update,
        R: NextInt,
    {
        match self {
            StatementKind::CompoundStatement(statement) => statement.execute(context),
        }
    }
}

/// Used in functions that return `StatementOutcome`; this macro executes the
/// statement and evaluates to the value. However, if exceptional control flow
/// occurred, this function will short-circuit and exit the current function
/// with the appropriate variant of `StatementOutcome`.
macro_rules! execute_with_ctrl_flow {
    ($statement:expr, $context:expr) => {{
        match $statement.execute($context) {
            StatementOutcome::Return(value) => value,
            StatementOutcome::EarlyReturn(value) => return StatementOutcome::EarlyReturn(value),
            StatementOutcome::Error(message) => return StatementOutcome::Error(message),
        }
    }};
}

pub struct CompoundStatement {
    pub statements: Vec<StatementKind>,
}

impl Statement for CompoundStatement {
    fn execute<'r, U, R>(&self, context: &mut ExecutionContext<'r, U, R>) -> StatementOutcome
    where
        U: Update,
        R: NextInt,
    {
        for statement in &self.statements {
            let _ = execute_with_ctrl_flow!(statement, context);
        }
        StatementOutcome::Return(None)
    }
}

pub struct ClearAll {}

impl Statement for ClearAll {
    fn execute<'r, U, R>(&self, context: &mut ExecutionContext<'r, U, R>) -> StatementOutcome
    where
        U: Update,
        R: NextInt,
    {
        context.executor.get_world().borrow_mut().clear_all();
        StatementOutcome::Return(None)
    }
}

pub struct CreateTurtles {
    /// The statement to evaluate to determine how many turtles to create.
    count: StatementKind,
    /// The name of the breed of turtles to create.
    breed_name: Rc<str>,
    /// The statements to run for each turtle after they are created.
    commands: Option<CompoundStatement>,
}

impl Statement for CreateTurtles {
    fn execute<'r, U, R>(&self, context: &mut ExecutionContext<U, R>) -> StatementOutcome
    where
        U: Update,
        R: NextInt,
    {
        // TODO must be executed by the observer only

        // evaluate the count argument
        let count = execute_with_ctrl_flow!(self.count, context);
        let Some(count) = count else {
            return StatementOutcome::Error(value::String::from(
                "input to `create-turtles` must return a value",
            ));
        };
        let Some(count) = count.to_u64_round_to_zero() else {
            return StatementOutcome::Error(value::String::from(
                "`create-turtles` expected this input to be a number",
            ));
        };

        // create the turtles
        let new_turtles = {
            let mut new_turtles = Vec::new();
            let world = context.executor.get_world();
            world.borrow_mut().turtle_manager.create_turtles(
                count,
                &self.breed_name,
                0.0, // TODO magic numbers
                0.0,
                |turtle| {
                    new_turtles.push(turtle.clone());
                    context
                        .updater
                        .update_turtle(&turtle.borrow(), FlagSet::default());
                },
                context.next_int,
            );
            new_turtles
        };

        // run the commands on the turtles
        if let Some(commands) = &self.commands {
            // set up the proper asker/executor for the commands
            let my_asker = mem::replace(&mut context.asker, context.executor.clone());

            for turtle in new_turtles {
                context.executor = Agent::Turtle(turtle.clone());
                let _ = execute_with_ctrl_flow!(commands, context);
            }

            // restore the asker/executor
            context.executor = mem::replace(&mut context.asker, my_asker);
        }

        // create-turtles does not return anything
        StatementOutcome::Return(None)
    }
}

pub struct SetTurtleSize {
    /// The statement to evaluate to determine the size to set.
    size: StatementKind,
}

impl Statement for SetTurtleSize {
    fn execute<'r, U, R>(&self, context: &mut ExecutionContext<'r, U, R>) -> StatementOutcome
    where
        U: Update,
        R: NextInt,
    {
        // evaluate the size argument
        let Some(size) = execute_with_ctrl_flow!(self.size, context) else {
            return StatementOutcome::Error(value::String::from(
                "input to `set-turtle-size` must return a value",
            ));
        };
        let Some(size) = size.to_f64() else {
            return StatementOutcome::Error(value::String::from(
                "`set-turtle-size` expected this input to be a number",
            ));
        };

        // set the size of the turtle
        if let Agent::Turtle(turtle) = &context.executor {
            turtle.borrow_mut().set_size(size);
            context
                .updater
                .update_turtle(&turtle.borrow(), FlagSet::default());
            StatementOutcome::Return(None)
        } else {
            StatementOutcome::Error(value::String::from(
                "`set-turtle-size` can only be executed by a turtle",
            ))
        }
    }
}
