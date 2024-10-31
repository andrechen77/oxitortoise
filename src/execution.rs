use std::{fmt::Debug, mem, rc::Rc};

use crate::{agent::Agent, updater::Update, value::{self, Value}};

mod manager;

use flagset::FlagSet;
pub use manager::ProcedureManager;

pub struct CommandProcedure<U> {
    pub name: Rc<str>,
    pub start: i32, // TODO consider making (start, end) its own type indicating a NetLogo source range
    pub end: i32,
    pub action: CompoundStatement<U>,
}

impl<U> Debug for CommandProcedure<U> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command")
            .field("name", &self.name)
            .field("start", &self.start)
            .field("end", &self.end)
            .finish_non_exhaustive()
    }
}

/// Holds
struct ExecutionContext<U> {
    /// The agent that is executing the current command. The `self` reporter in
    /// NetLogo returns this value if it is not the observer.
    executor: Agent,
    /// The agent that asked the executor to execute the current command. The
    /// `myself` reporter in NetLogo returns this value if it is not the
    /// observer.
    asker: Agent,
    /// The output for all updates that occur during execution.
    updater: U,
    /// `None` if not inside the catch block of a `carefully`; this represents
    /// the message of the error currently being handled.
    error: Option<value::String>,
    // TODO rngs, etc.
}

enum StatementOutcome {
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
trait Statement<U: Update> {
    /// Executes the statement with the specified execution context and returns
    /// the outcome. All updates that the statement would like to propagate
    /// must go into the updater of the given execution context. As as general
    /// rule of good practice, a statement should be able to assume that all
    /// the state in its execution context (which uses RefCell heavily) is
    /// currently unborrowed, so a statement should make sure that it stops
    /// borrowing that state when it calls other statements.
    fn execute(&self, context: &mut ExecutionContext<U>) -> StatementOutcome;
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

struct CompoundStatement<U> {
    pub statements: Vec<Box<dyn Statement<U>>>,
}

impl<U: Update> Statement<U> for CompoundStatement<U> {
    fn execute(&self, context: &mut ExecutionContext<U>) -> StatementOutcome {
        for statement in &self.statements {
            let _ = execute_with_ctrl_flow!(statement, context);
        }
        StatementOutcome::Return(None)
    }
}

struct ClearAll {}

impl<U: Update> Statement<U> for ClearAll {
    fn execute(&self, context: &mut ExecutionContext<U>) -> StatementOutcome {
        context.executor.get_world().borrow_mut().clear_all();
        StatementOutcome::Return(None)
    }
}

struct CreateTurtles<U> {
    /// The statement to evaluate to determine how many turtles to create.
    count: Box<dyn Statement<U>>,
    /// The name of the breed of turtles to create.
    breed_name: Rc<str>,
    /// The statements to run for each turtle after they are created.
    commands: Option<CompoundStatement<U>>,
}

impl <U: Update> Statement<U> for CreateTurtles<U> {
    fn execute(&self, context: &mut ExecutionContext<U>) -> StatementOutcome {
        // TODO must be executed by the observer only

        // evaluate the count argument
        let count = execute_with_ctrl_flow!(self.count, context);
        let Some(count) = count else {
            return StatementOutcome::Error(value::String::from("input to `create-turtles` must return a value"));
        };
        let Some(count) = count.to_u64_round_to_zero() else {
            return StatementOutcome::Error(value::String::from("`create-turtles` expected this input to be a number"));
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
                    context.updater.update_turtle(&turtle.borrow(), FlagSet::default());
                }
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
