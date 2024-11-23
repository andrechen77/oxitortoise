use crate::{
    execution::{
        monolithic::ExecuteMonolith,
        ExecutionContext,
    },
    rng::NextInt,
    updater::Update,
    value::PolyValue,
};


pub struct ClearAllMonolith {}

impl ExecuteMonolith for ClearAllMonolith {
    fn execute_monolith<'w, U, R>(
        &self,
        context: &mut ExecutionContext<'w, U, R>,
    ) -> PolyValue
    where
        U: Update,
        R: NextInt,
    {
        context.world.clear_all();
        PolyValue::ERASED
    }
}