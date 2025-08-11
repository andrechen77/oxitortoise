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

use std::ops::{Index, IndexMut};

use crate::stackify;

pub struct Program {
    pub entrypoints: Vec<FunctionId>,
    pub functions: Vec<Function>,
}

pub struct Function {
    pub parameter_types: Vec<ValType>,
    pub return_types: Vec<ValType>,
    pub instructions: Vec<InsnKind>,
}

pub struct FunctionId;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct InsnRef(usize);

pub struct ImportedFunctionId {
    name: &'static str,
}

/// A machine-level type. These are just numbers that have no higher-level
/// semantic meaning.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValType {
    I8,
    I16,
    I32,
    I64,
    F64,
    Ptr,
    FnPtr,
}

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
        multivalue: InsnRef,
        index: u32,
    },
    /// Add a compile-time offset to a pointer, producing a pointer to a
    /// subfield.
    DeriveField {
        offset: usize,
        ptr: InsnRef,
    },
    /// Add a dynamic offset to a pointer, producing a pointer to an element of
    /// an array.
    DeriveElement {
        element_size: usize,
        ptr: InsnRef,
        index: InsnRef,
    },
    /// Load a value from memory.
    ///
    /// For loading values from the current function's stack frame, use [`InstructionKind::StackLoad`]
    MemLoad {
        r#type: ValType,
        offset: usize,
        ptr: InsnRef,
    },
    /// Store a value into memory.
    ///
    /// For storing values onto the current function's stack frame, use [`InstructionKind::StackStore`]
    MemStore {
        offset: usize,
        ptr: InsnRef,
        value: InsnRef,
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
        value: InsnRef,
    },
    CallImportedFunction {
        function: ImportedFunctionId,
        args: Box<[InsnRef]>,
    },
    CallUserFunction {
        function: FunctionId,
        args: Box<[InsnRef]>,
    },
    UnaryOp {
        op: UnaryOpcode,
        operand: InsnRef,
    },
    BinaryOp {
        /// The operation to perform. This also determines the types of the
        /// inputs and outputs.
        op: BinaryOpcode,
        lhs: InsnRef,
        rhs: InsnRef,
    },
    /// Break out of some number of control flow constructs. A depth of 0 means
    /// breaking out of the current construct. A depth of 1 means breaking out
    /// of the current construct and the enclosing construct, etc. This is also
    /// used to implement early returns.
    Break {
        depth: u16,
        values: Vec<InsnRef>,
    },
    /// A breakable block.
    Block {
        /// The number of instructions in the block's body. The following
        /// `body_len` instructions are considered inside this instruction.
        body_len: usize,
    },
    /// An if-else statement.
    IfElse {
        condition: InsnRef,
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
}

pub enum UnaryOpcode {
    I64ToI32,
}

pub enum BinaryOpcode {
    Add,
    Sub,
    Lt,
    Eq,
}

pub struct InsnRefIter<'a> {
    next: InsnRef,
    instructions: &'a [InsnKind],
}

impl<'a> Iterator for InsnRefIter<'a> {
    type Item = InsnRef;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next.0 >= self.instructions.len() {
            return None;
        }

        let return_value = self.next;
        // calculate the next instruction to visit based on what the returned
        // instruction was
        self.next.0 += match self.instructions[self.next.0] {
            InsnKind::Block { body_len } => 1 + body_len,
            InsnKind::IfElse {
                then_len, else_len, ..
            } => 1 + then_len + else_len,
            InsnKind::Loop { body_len } => 1 + body_len,
            _ => 1,
        };
        Some(return_value)
    }
}
