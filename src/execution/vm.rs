use std::{cell::RefCell, rc::Rc};

use crate::{
    sim::{agent::AgentId, value::PolyValue, world::World},
    util::rng::CanonRng,
    updater::Update,
};

use instruction::Instruction;
use structure::StructureFrame;

mod functionality;
pub mod instruction;
pub mod structure;

pub struct ExecutionContext<'w, U> {
    /// The world in which the execution is occurring.
    pub world: &'w mut World,
    /// The agent that is executing the current command. The `self` reporter in
    /// NetLogo returns this value if it is not the observer.
    pub executor: AgentId,
    /// The agent that asked the executor to execute the current command. The
    /// `myself` reporter in NetLogo returns this value if it is not the
    /// observer.
    pub asker: AgentId,
    /// A stack of frames that track the current location of execution, like a
    /// call stack but encompassing structured programming constructs other than
    /// just function calls. From top to bottom, the frame stack holds
    /// information about each nested construct.
    pub frame_stack: Vec<StructureFrame>,
    /// A stack of values used by the non-opaque commands to remember values
    /// during a thread of execution.
    pub operand_stack: Vec<PolyValue>,
    /// The next instruction to execute. This is null if there is no next
    /// instruction to execute; this means an instruction can set this value
    /// to null to halt execution.
    ///
    /// # Safety
    ///
    /// When assigning to this value, ensure that the value is either null or a
    /// valid pointer that can be dereferenced to `&Instruction`. The pointer
    /// should be valid for as long as the execution context exists.
    pub next_instruction: *const Instruction,
    /// The output for all updates that occur during execution.
    pub updater: U,
    pub next_int: Rc<RefCell<CanonRng>>,
    // TODO rngs, etc.
}

pub trait Execute<U> {
    /// # Safety
    ///
    /// Implementors are allowed to do unsafe operations in this method within
    /// certain constraits. Callers must ensure that these constraits are
    /// satisfied when calling this method.
    ///
    /// - The instructions must be laid out where every instruction reachable
    /// from the starting instruction (`context.next_instruction`) is in an
    /// array, and every instruction that can fall through in the array is
    /// followed by another instruction in that array (except for halt and
    /// unconditional jumps). Also every instruction except for halts and
    /// unconditional jumps must set the next_instruction to point to a valid
    /// instruction under these rules, while of course following the borrowing
    /// rules.
    /// - Some instructions may assume that the operand stack has values with
    /// some type at the top. These assumptions should be documented on the
    /// struct representating that instruction, and that instruction may
    /// unsafely get those values without type-checking.
    unsafe fn execute<'w>(&self, context: &mut ExecutionContext<'w, U>);
}

/// # Safety
///
/// See the safety section of [`Execute::execute`].
pub unsafe fn execute_loop<'w, U: Update>(context: &mut ExecutionContext<'w, U>) {
    while !context.next_instruction.is_null() {
        // SAFETY: see precondition
        let instruction = unsafe { &*context.next_instruction };
        context.next_instruction = unsafe { context.next_instruction.offset(1) };

        instruction.execute(context);
    }
}
