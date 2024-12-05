use std::{cell::RefCell, collections::VecDeque, ops::DerefMut as _, rc::Rc};

use crate::{
    agent::AgentId,
    execution::vm::{
        instruction::Instruction,
        structure::{pop_until_structure_frame, StructureFrame},
        Execute, ExecutionContext,
    },
    rng::CanonRng,
    shuffle_iterator::ShuffledOwned,
    topology::Point,
    turtle::TurtleId,
    unsafe_getter::{unwrap_option, unwrap_polyvalue},
    value::{self},
};

/// Creates turtles and sets up initialization for each turtle.
///
/// # Preconditions and Postconditions
///
/// ## Arguments
///
/// From first to last (bottommost to topmost), the elements on the operand
/// stack must be:
/// - the number of turtles to create ([`value::Float`])
///
/// ## Child Instructions
///
/// A `create-turtles` construct may execute some instructions for each created
/// turtle. The instructions following this instruction up to the next executed
/// `ContinueCrtAsk` are run with each created turtle as an executor. An
/// `CrtAskFrame` is pushed onto to the frame stack to track the remaining
/// turtles to execute these instructions for. This instruction will
/// automatically set the first turtle as the executor for the following
/// instructions; to continue iteration to future turtles, encountering a
/// `ContinueCrtAsk` instruction gets the next turtle from that list and runs
/// these instructions for that turtle. If there are no turtles left to iterate,
/// either this instruction or `ContinueCrtAsk` will pop the `CrtAskFrame` and
/// jump to the instruction after this construct.
///
/// ## Return
///
/// This instruction leaves no extra operands on the stack.
#[derive(Debug)]
pub struct CreateTurtlesWithCommands {
    /// The name of the breed of turtles to create.
    breed_name: Rc<str>,
    /// The number of instructions after this instruction that are considered
    /// the "body" of the executed commands for this instruction. Control flow
    /// will jump to `body_len` instructions after this instruction when
    /// all turtles have run the initialization instructions.
    body_len: usize,
}

#[derive(Debug)]
pub struct CrtAskFrame {
    /// The turtles that have not yet been initialized.
    remaining_turtles: ShuffledOwned<TurtleId, Rc<RefCell<CanonRng>>>,
    /// The first instruction to jump to when initializing a turtle.
    body_start: *const Instruction,
    /// The first instruction after the initialization instructions.
    body_len: usize,
    /// The length of the operand stack when this frame was pushed. Exiting this
    /// structure will truncate the operand stack to this length.
    operand_stack_len: usize,
}

impl<U> Execute<U> for CreateTurtlesWithCommands {
    unsafe fn execute<'w>(&self, context: &mut ExecutionContext<'w, U>) {
        // this is an observer command only
        debug_assert!(context.executor == AgentId::Observer);
        debug_assert!(context.asker == AgentId::Observer);

        // get the count argument
        let count = unwrap_option(context.operand_stack.pop());
        let count: value::Float = unwrap_polyvalue(count);
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

        // push the `CrtAskFrame` onto the frame stack
        context
            .frame_stack
            .push(StructureFrame::CrtAsk(CrtAskFrame {
                remaining_turtles: ShuffledOwned::new(
                    VecDeque::from(new_turtles),
                    Rc::clone(&context.next_int),
                ),
                body_start: context.next_instruction,
                body_len: self.body_len,
                operand_stack_len: context.operand_stack.len(),
            }));

        // initialize the first turtle
        start_ask_next_turtle(context);
    }
}

pub struct ContinueCrtAsk {}

impl<U> Execute<U> for ContinueCrtAsk {
    unsafe fn execute<'w>(&self, context: &mut ExecutionContext<'w, U>) {
        start_ask_next_turtle(context);
    }
}

fn start_ask_next_turtle<'w, U>(context: &mut ExecutionContext<'w, U>) {
    let crt_ask_frame: &mut CrtAskFrame =
        unwrap_option(pop_until_structure_frame(&mut context.frame_stack));

    // pop extra values from the operand stack
    debug_assert!(context.operand_stack.len() >= crt_ask_frame.operand_stack_len);
    context
        .operand_stack
        .truncate(crt_ask_frame.operand_stack_len);

    if let Some(turtle) = crt_ask_frame.remaining_turtles.next() {
        // switch to the next turtle
        context.executor = AgentId::Turtle(turtle);
        context.next_instruction = crt_ask_frame.body_start;
    } else {
        // exit from the ask body
        context.next_instruction = unsafe { crt_ask_frame.body_start.add(crt_ask_frame.body_len) };
        context.frame_stack.pop();
    }
}
