//! Low-level intermediate representation, intended to be converted to
//! platform-specific code (e.g. Wasm, CLIF).
//!
//! # Instructions
//!
//! Each instruction in a function takes some number of inputs and some number
//! of outputs. Instructions can use the outputs of previous instructions as
//! their own inputs, created a graph structure, almost like sea-of-nodes.
//! Control flow is structured (rather than a control flow graph), being handled
//! by special instructions for blocks, if-else, and loops. These instructions
//! may "claim" a region of instructions immediately following it; these
//! following instructions are considered "inside" the control flow instruction.
//! In addition to instructions only being able to use previous instructions as
//! inputs, instructions also may not reach *into* another control flow
//! construct, though it can reach *out* of its own and any recursively
//! containing control flow constructs.
//!
//! Instructions are identified by a unique ID, which is their index from the
//! start of the function.
//!
//! ## Control Flow Instructions
//!
//! Blocks, loops, and if-else statements mirror the semantics of corresponding
//! constructs in WebAssembly (because we want to compile to WebAssembly).
//!
//! A block is a region of code that may return a value. The return value of the
//! block is encoded by the output of the corresponding `BlockEnd` instruction.
//! As such, the output of an `EndBlock` instruction *may not* be used inside
//! the block it represents.
//!
//! A loop is a region of code that may use different values on each iteration
//! and may return a value. The values that may change over iterations of the
//! loop are encoded by the output of the `LoopBody` instruction, which *may* be
//! used inside the loop it represents. The return value of the loop is encoded
//! by the output of the `LoopEnd` instruction, which *may not* be used inside
//! the loop it represents.
//!
//! TODO is the loop end instruction really necessary? can't we just directly
//! use the output of instructions inside the loop?
//!
//! An if-else is two distinct regions of code, exactly one of which will be
//! executed every time the if-else itself is executed, which may return a
//! value. The return value of the if-else is encoded by the output of the
//! `IfElseEnd` instruction, which *may not* be used inside the if-else.

// TODO decide on the names of the block, loop, and if-else instructiohns

use std::ops::{Add, AddAssign, Range};

use derive_more::{From, Into};
use smallvec::{SmallVec, ToSmallVec as _, smallvec};
use typed_index_collections::{TiSlice, TiVec};

#[macro_use]
mod macros;
pub use macros::instructions;

#[derive(Debug, PartialEq, Eq)]
pub struct Program {
    pub entrypoints: Vec<FunctionId>,
    pub functions: Vec<Function>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Function {
    pub parameter_types: Vec<ValType>,
    pub return_types: Vec<ValType>,
    pub instructions: TiVec<InsnPc, InsnKind>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FunctionId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Into, From)]
pub struct InsnPc(usize);
impl Add<usize> for InsnPc {
    type Output = InsnPc;

    fn add(self, rhs: usize) -> Self::Output {
        InsnPc(self.0 + rhs)
    }
}
impl AddAssign<usize> for InsnPc {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ImportedFunctionId {
    name: &'static str,
}

/// A machine-level type. These are just numbers that have no higher-level
/// semantic meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ValType {
    I8,
    I16,
    I32,
    I64,
    F64,
    Ptr,
    FnPtr,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InsnKind {
    Argument {
        r#type: ValType,
        index: u32,
    },
    Const {
        r#type: ValType,
        /// The bit pattern of the value to store.
        value: i64,
    },
    /// Given the output of an instruction that takes multiple values, project
    /// a single value.
    Project {
        multivalue: InsnPc,
        index: u32,
    },
    /// Add a compile-time offset to a pointer, producing a pointer to a
    /// subfield.
    DeriveField {
        offset: usize,
        ptr: InsnPc,
    },
    /// Add a dynamic offset to a pointer, producing a pointer to an element of
    /// an array.
    DeriveElement {
        element_size: usize,
        ptr: InsnPc,
        index: InsnPc,
    },
    /// Load a value from memory.
    ///
    /// For loading values from the current function's stack frame, use [`InstructionKind::StackLoad`]
    MemLoad {
        r#type: ValType,
        offset: usize,
        ptr: InsnPc,
    },
    /// Store a value into memory.
    ///
    /// For storing values onto the current function's stack frame, use [`InstructionKind::StackStore`]
    MemStore {
        offset: usize,
        ptr: InsnPc,
        value: InsnPc,
    },
    /// Load a value from the stack.
    StackLoad {
        r#type: ValType,
        /// The offset from the top of the stack at which to load the value.
        offset: usize,
    },
    /// Store a value onto the stack.
    StackStore {
        /// The offset from the top of the stack at which to store the value.
        offset: usize,
        value: InsnPc,
    },
    CallImportedFunction {
        function: ImportedFunctionId,
        args: Box<[InsnPc]>,
    },
    CallUserFunction {
        function: FunctionId,
        args: Box<[InsnPc]>,
    },
    UnaryOp {
        op: UnaryOpcode,
        operand: InsnPc,
    },
    BinaryOp {
        /// The operation to perform. This also determines the types of the
        /// inputs and outputs.
        op: BinaryOpcode,
        lhs: InsnPc,
        rhs: InsnPc,
    },
    /// Break out of some number of control flow constructs. A depth of 0 means
    /// breaking out of the current construct. A depth of 1 means breaking out
    /// of the current construct and the enclosing construct, etc. This is also
    /// used to implement early returns.
    Break {
        depth: u16,
        values: Box<[InsnPc]>,
    },
    /// Conditionally break out of some number of control flow constructs. See
    /// [`InsnKind::Break`] for more information.
    ConditionalBreak {
        condition: InsnPc,
        depth: u16,
        values: Box<[InsnPc]>,
    },
    /// A breakable block.
    Block {
        /// The number of instructions in the block's body. The following
        /// `body_len` instructions are considered inside this instruction.
        body_len: usize,
    },
    /// An if-else statement.
    IfElse {
        condition: InsnPc,
        /// The number of instructions in the then branch. The following
        /// `then_len` instructions are considered inside this instruction.
        then_len: usize,
        /// The number of instructions in the else branch. The following
        /// `then_len..then_len + else_len` instructions are considered inside
        /// this instruction.
        else_len: usize,
    },
    /// A loop.
    Loop {
        /// The number of instructions in the loop's body. The following
        /// `body_len` instructions are considered inside this instruction.
        body_len: usize,
    },
    /// An argument in a loop body
    LoopArgument {
        /// The initial value of the argument when the loop is entered.
        /// This is not considered an input to this instruction.
        initial_value: InsnPc,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum UnaryOpcode {
    I64ToI32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BinaryOpcode {
    Add,
    Sub,
    Lt,
    Eq,
}

impl InsnKind {
    /// Returns all nested instruction sequences inside this instruction, as
    /// well as the program counter of the instruction that follows.
    pub fn extent(&self, my_pc: InsnPc) -> (SmallVec<[Range<InsnPc>; 2]>, InsnPc) {
        match self {
            InsnKind::Block { body_len } => {
                let body_start = my_pc + 1;
                let body_end = body_start + *body_len;
                (smallvec![body_start..body_end], body_end)
            }
            InsnKind::Loop { body_len } => {
                let body_start = my_pc + 1;
                let body_end = body_start + *body_len;
                (smallvec![body_start..body_end], body_end)
            }
            InsnKind::IfElse {
                then_len, else_len, ..
            } => {
                let then_start = my_pc + 1;
                let then_end = then_start + *then_len;
                let else_end = then_end + *else_len;
                (
                    smallvec![then_start..then_end, then_end..else_end],
                    else_end,
                )
            }
            _ => (smallvec![], my_pc + 1),
        }
    }

    pub fn inputs(&self) -> SmallVec<[InsnPc; 2]> {
        match self {
            InsnKind::Argument { .. } => smallvec![],
            InsnKind::Const { .. } => smallvec![],
            InsnKind::Project { multivalue, .. } => smallvec![*multivalue],
            InsnKind::DeriveField { ptr, .. } => smallvec![*ptr],
            InsnKind::DeriveElement { ptr, index, .. } => smallvec![*ptr, *index],
            InsnKind::MemLoad { ptr, .. } => smallvec![*ptr],
            InsnKind::MemStore { ptr, value, .. } => smallvec![*ptr, *value],
            InsnKind::StackLoad { .. } => smallvec![],
            InsnKind::StackStore { value, .. } => smallvec![*value],
            InsnKind::CallImportedFunction { args, .. } => args.to_smallvec(),
            InsnKind::CallUserFunction { args, .. } => args.to_smallvec(),
            InsnKind::UnaryOp { operand, .. } => smallvec![*operand],
            InsnKind::BinaryOp { lhs, rhs, .. } => smallvec![*lhs, *rhs],
            InsnKind::Break { values, .. } => values.to_smallvec(),
            InsnKind::ConditionalBreak {
                condition, values, ..
            } => {
                let mut inputs = values.to_smallvec();
                inputs.push(*condition);
                inputs
            }
            InsnKind::Block { .. } => smallvec![],
            InsnKind::IfElse { condition, .. } => smallvec![*condition],
            InsnKind::Loop { .. } => smallvec![],
            InsnKind::LoopArgument { initial_value: _ } => smallvec![],
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InsnRefIter<'a> {
    remaining: Range<InsnPc>,
    instructions: &'a TiSlice<InsnPc, InsnKind>,
}

impl<'a> InsnRefIter<'a> {
    // Take an iterator to the entire vector instead of just a slice, to enforce
    // that we have all the instructions.
    pub fn new_with_range(instructions: &'a TiVec<InsnPc, InsnKind>, range: Range<InsnPc>) -> Self {
        Self {
            remaining: range,
            instructions,
        }
    }

    // Take an iterator to the entire vector instead of just a slice, to enforce
    // that we have all the instructions.
    pub fn new(instructions: &'a TiVec<InsnPc, InsnKind>) -> Self {
        Self::new_with_range(
            instructions,
            InsnPc::from(0)..InsnPc::from(instructions.len()),
        )
    }
}

impl<'a> Iterator for InsnRefIter<'a> {
    type Item = (SmallVec<[Range<InsnPc>; 2]>, InsnPc);

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }

        let next_pc = self.remaining.start;
        // calculate the next instruction to visit based on what the returned
        // instruction was
        let (inner_seqs, succ) = self.instructions[next_pc].extent(next_pc);
        self.remaining.start = succ;
        Some((inner_seqs, next_pc))
    }
}
