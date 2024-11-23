use crate::{
    execution::{
        monolithic::{ExecuteMonolith, StatementMonolith},
        ExecutionContext,
    },
    rng::NextInt,
    updater::Update,
    value::PolyValue,
};

pub struct CompoundStatementMonolith {
    pub statements: Vec<StatementMonolith>,
}

impl ExecuteMonolith for CompoundStatementMonolith {
    fn execute_monolith<'w, U, R>(
        &self,
        context: &mut ExecutionContext<'w, U, R>,
    ) -> PolyValue
    where
        U: Update,
        R: NextInt,
    {
        for statement in &self.statements {
            let _ = statement.execute_monolith(context);
        }
        PolyValue::ERASED
    }
}
