use crate::{rng::NextInt, updater::Update, value::PolyValue};

use super::{
    pieces::{ClearAllMonolith, CompoundStatementMonolith, CreateTurtlesMonolith},
    ExecutionContext,
};

/// This trait characterizes NetLogo code that can be executed to completion
/// without the caller having to "catch" exceptions or early returns. The caller
/// simply calls the code (through the [`ExecuteMonolith::execute_monolith`]
/// method) and receives a value in return.
pub trait ExecuteMonolith {
    /// Executes the statement with the specified execution context and returns
    /// the outcome. Updates that the statement would like to propagate go into
    /// the updater of the given execution context. As a general rule of good
    /// practice, a statement should be able to assume that all the state in its
    /// execution context is currently unborrowed (in the sense of
    /// dynamically-checked borrowing, namely `RefCell`), so a statement should
    /// make sure that it stops borrowing that state when it calls other
    /// statements.
    fn execute_monolith<'w, U, R>(&self, context: &mut ExecutionContext<'w, U, R>) -> PolyValue
    where
        U: Update,
        R: NextInt;
}

/// An enum covering all possible kinds of opaque statements.
// Because the size of an enum is the size of its largest variant, we should
// aim to keep all variants small; larger variants should be boxed.
// TODO add all variants
pub enum StatementMonolith {
    Compound(CompoundStatementMonolith),
    ClearAll(ClearAllMonolith),
    CreateTurtles(Box<CreateTurtlesMonolith>),
}

impl ExecuteMonolith for StatementMonolith {
    fn execute_monolith<'w, U, R>(&self, context: &mut ExecutionContext<'w, U, R>) -> PolyValue
    where
        U: Update,
        R: NextInt,
    {
        match self {
            StatementMonolith::Compound(statement) => statement.execute_monolith(context),
            StatementMonolith::ClearAll(statement) => statement.execute_monolith(context),
            StatementMonolith::CreateTurtles(statement) => statement.execute_monolith(context),
        }
    }
}
