use std::fmt::Debug;
use std::rc::Rc;

use super::vm::instruction::Instruction;

#[derive(Debug)]
pub struct Procedure {
    name: Rc<str>,
    instructions: Vec<Instruction>,
}

impl Procedure {
    /// # Safety
    ///
    /// See [`here`](super::vm::Execute::execute) for invariants that must be
    /// upheld by the instructions.
    pub unsafe fn new(name: Rc<str>, instructions: Vec<Instruction>) -> Self {
        Self { name, instructions }
    }

    pub fn name(&self) -> &Rc<str> {
        &self.name
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }
}
