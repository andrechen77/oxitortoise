//! Low-level intermediate representation, intended to be converted to
//! platform-specific code (e.g. Wasm, CLIF).
//!
//! # Instructions
//!
//! Each instruction in a function takes some number of inputs and has some
//! number of outputs. Instructions can use the outputs of previous instructions
//! as their own inputs, created a graph structure, almost like sea-of-nodes.
//! The function body is a single instruction sequence, but instruction
//! sequences can "contain" other instruction sequences through compound
//! instructions. Control flow is structured (rather than a control flow graph),
//! being handled by special instructions for blocks, if-else, and loops. In
//! addition to instructions only being able to use previous instructions as
//! inputs, instructions also may not reach *into* another control flow
//! construct, though it can reach *out* of its own and any recursively
//! containing control flow constructs.
//!
//! Instructions are identified by an index within their containing sequence,
//! this is their [`InsnIdx`]. Instruction sequences also have their own unique
//! ID, [`InsnSeqId`]. An instruction sequence ID and an index together uniquely
//! identify an instruction, known as the [`InsnPc`].
//!
//! A separate ID called [`ValRef`] identifies outputs of instructions. Some
//! instructions have no outputs and therefore do not get any `ValRef`s; others
//! may have multiple and would get multiple `ValRef`s.
//!
//! ## Control Flow Instructions
//!
//! Blocks, loops, and if-else statements mirror the semantics of corresponding
//! constructs in WebAssembly (because we want to compile to WebAssembly).
//!
//! A block is a region of code that may return a value. Breaking within a block
//! will exit the block and cause it to return the value that was broken with.
//! An if-else, semantically, is just two blocks, only one of which will be
//! entered. Breaking from an if-else will exit the if-else and cause it to
//! return the value that was broken with.
//!
//! A loop is a region of code that may use different values on each iteration
//! and may return a value. A loop instruction may take inputs; these are used
//! to initialize iteration values that might change between loop iterations.
//! Within the loop body, these are accessible with the Breaking within a loop
//! will exit the current iteration of the loop body and re-enter the loop body
//! with the loop iteration values set to the values that were broken with.

use std::{collections::HashMap, iter::Step};

use derive_more::{From, Into};
use typed_index_collections::{TiSlice, TiVec};

#[macro_use]
mod macros;
pub use macros::lir_function;

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Program {
    pub entrypoints: Vec<FunctionId>,
    pub user_functions: TiVec<FunctionId, Function>,
    pub imported_functions: TiVec<ImportedFunctionId, ImportedFunction>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ImportedFunction {
    pub parameter_types: Vec<ValType>,
    pub return_type: Vec<ValType>,
    /// Import the function by this name in Wasm.
    pub name: &'static str,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Function {
    pub parameter_types: Vec<ValType>,
    pub body: Block,
    pub insn_seqs: TiVec<InsnSeqId, TiVec<InsnIdx, InsnKind>>,
    /// The number of bytes to allocate on the stack for this function.
    pub stack_space: usize,
}

#[derive(Debug, PartialEq, Eq, Into, From, Hash)]
pub struct FunctionId(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Into, From)]
pub struct InsnIdx(pub usize);
impl Step for InsnIdx {
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        (end.0 - start.0, Some(end.0 - start.0))
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        Some(InsnIdx(start.0.checked_add(count)?))
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        Some(InsnIdx(start.0.checked_sub(count)?))
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, From, Into)]
pub struct ImportedFunctionId(pub usize);

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

// TODO remove this and just use a SmallVec<[ValType; 1]>
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InsnOutput {
    /// The instruction outputs a single value,
    Single(ValType),
    /// The instruction outputs some other number of values. This may be zero.
    Other(Vec<ValType>),
}

impl InsnOutput {
    pub fn from_types_array<const N: usize>(types: [ValType; N]) -> Self {
        if N == 1 { Self::Single(types[0]) } else { Self::Other(types.to_vec()) }
    }

    pub fn from_types_iter(types: impl IntoIterator<Item = ValType>) -> Self {
        let types = types.into_iter().collect::<Vec<_>>();
        if types.len() == 1 { Self::Single(types[0]) } else { Self::Other(types) }
    }

    pub fn index(&self, index: u8) -> ValType {
        self.as_ref()[index as usize]
    }
}

impl AsRef<[ValType]> for InsnOutput {
    fn as_ref(&self) -> &[ValType] {
        match self {
            InsnOutput::Single(ty) => std::slice::from_ref(ty),
            InsnOutput::Other(tys) => tys.as_slice(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct InsnPc(pub InsnSeqId, pub InsnIdx);

/// A reference to a value produced by an instruction. Starts from 0 and counts
/// up for each value produced by an instruction in the function. Some
/// instructions may produce multiple values, while others may produce zero.
#[derive(Debug, PartialEq, Eq, /* PartialOrd, Ord, */ Hash, Clone, Copy)]
pub struct ValRef(pub InsnPc, pub u8);

pub type InsnSeq = TiSlice<InsnIdx, InsnKind>;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, From, Into)]
pub struct InsnSeqId(pub usize);

#[derive(Debug, PartialEq, Eq)]
pub enum InsnKind {
    /// Outputs all arguments of the function.
    FunctionArgs {
        output_type: InsnOutput,
    },
    // TODO rethink loop args. first, they should output a multivalue, instead
    // of being a single value, since that's what it will look like on the stack
    // machine. In addition, should the loop args instruction be considered to
    // output values on the stack, or does the instruction only exist so that
    // other instructions can have a ValRef to use the loop args? If the latter,
    // then I need to modify the stackify_lir code so that entering a loop body
    // *doesn't* start with a fresh stack. If the former, then I'll have to deal
    // with the fact that every loop body must start with a loop args
    // instruction.
    /// Outputs all arguments of a loop body.
    LoopArg {
        /// The initial value of the arguments when the loop is entered.
        /// This is not considered an input to this instruction.
        initial_value: ValRef,
    },
    Const {
        r#type: ValType,
        /// The bit pattern of the value to store.
        value: u64,
    },
    /// Add a compile-time offset to a pointer, producing a pointer to a
    /// subfield.
    DeriveField {
        offset: usize,
        ptr: ValRef,
    },
    /// Add a dynamic offset to a pointer, producing a pointer to an element of
    /// an array.
    DeriveElement {
        element_size: usize,
        ptr: ValRef,
        index: ValRef,
    },
    /// Load a value from memory.
    ///
    /// For loading values from the current function's stack frame, use [`InstructionKind::StackLoad`]
    MemLoad {
        r#type: ValType,
        offset: usize,
        ptr: ValRef,
    },
    /// Store a value into memory.
    ///
    /// For storing values onto the current function's stack frame, use [`InstructionKind::StackStore`]
    MemStore {
        offset: usize,
        ptr: ValRef,
        value: ValRef,
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
        value: ValRef,
    },
    CallImportedFunction {
        function: ImportedFunctionId,
        output_type: InsnOutput,
        args: Box<[ValRef]>,
    },
    CallUserFunction {
        function: FunctionId,
        output_type: InsnOutput,
        args: Box<[ValRef]>,
    },
    UnaryOp {
        op: UnaryOpcode,
        operand: ValRef,
    },
    BinaryOp {
        /// The operation to perform. This also determines the types of the
        /// inputs and outputs.
        op: BinaryOpcode,
        lhs: ValRef,
        rhs: ValRef,
    },
    /// Break out of some number of control flow constructs. This must be the
    /// last instruction within an instruction sequence.
    ///
    /// # Target
    ///
    /// The target parameter is the [`InsnSeqId`] of the sequence to break out
    /// of. The continuation depends on the compound instruction that the
    /// instruction sequence belongs to. If it belongs to a block or if-else,
    /// then this will jump to the end of the block/if-else, returning the
    /// arguments; if it belongs to a loop, then this will jump to the start of
    /// the loop body with the new loop arguments. Since `InsnSeqId(0)` is
    /// always the entire function, that targt can be used to implement early
    /// returns.
    Break {
        target: InsnSeqId,
        values: Box<[ValRef]>,
    },
    /// Conditionally break out of control flow constructs. See
    /// [`InsnKind::Break`] for more information.
    ConditionalBreak {
        target: InsnSeqId,
        condition: ValRef,
        values: Box<[ValRef]>,
    },
    // TODO add a fallthrough instruction to allow returning values from a
    // loop without wrapping to the top of the loop again.
    Block(Block),
    IfElse(IfElse),
    Loop(Loop),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Block {
    /// The type of output of this block.
    pub output_type: InsnOutput,
    /// The instructions inside the block.
    pub body: InsnSeqId,
}

#[derive(Debug, PartialEq, Eq)]
pub struct IfElse {
    /// The type of output of this if-else.
    pub output_type: InsnOutput,
    /// The condition of the if-else.
    pub condition: ValRef,
    /// The instructions inside the then branch.
    pub then_body: InsnSeqId,
    /// The instructions inside the else branch.
    pub else_body: InsnSeqId,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Loop {
    /// The initial values of the loop body arguments.
    pub inputs: Vec<ValRef>,
    /// The type of output of this loop.
    pub output_type: InsnOutput,
    /// The instructions inside the loop.
    pub body: InsnSeqId,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnaryOpcode {
    I64ToI32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinaryOpcode {
    Add,
    Sub,
    Lt,
    Eq,
}

impl InsnKind {
    /// Returns the number of new names produced by this instruction. This is
    /// used to assign [`ValRef`]s to the outputs of instructions.
    pub fn num_outputs(&self) -> usize {
        match self {
            InsnKind::FunctionArgs { .. } => 1,
            InsnKind::LoopArg { .. } => 1,
            InsnKind::Const { .. } => 1,
            InsnKind::DeriveField { .. } => 1,
            InsnKind::DeriveElement { .. } => 1,
            InsnKind::MemLoad { .. } => 1,
            InsnKind::MemStore { .. } => 0,
            InsnKind::StackLoad { .. } => 1,
            InsnKind::StackStore { .. } => 0,
            InsnKind::UnaryOp { .. } => 1,
            InsnKind::BinaryOp { .. } => 1,
            InsnKind::Break { .. } => 0,
            InsnKind::ConditionalBreak { .. } => 0,
            InsnKind::Block(Block { output_type, .. })
            | InsnKind::IfElse(IfElse { output_type, .. })
            | InsnKind::Loop(Loop { output_type, .. })
            | InsnKind::CallImportedFunction { output_type, .. }
            | InsnKind::CallUserFunction { output_type, .. } => output_type.as_ref().len(),
        }
    }
}

pub trait LirVisitor {
    fn start_insn_seq(&mut self, id: InsnSeqId);

    fn end_insn_seq(&mut self, id: InsnSeqId);

    /// Visits an instruction. `next_val_ref` is the index of the first output
    /// of this instruction, if it outputs any; this is used for assertions to
    /// ensure that output counting was correct.
    fn visit_insn(&mut self, insn: &InsnKind, pc: InsnPc);
}

pub fn visit_insn_seq<V: LirVisitor>(visitor: &mut V, function: &Function) {
    // let mut next_val_ref = ValRef(0);
    visit_insn_seq_recursive(visitor, function, function.body.body /* &mut next_val_ref */);
    fn visit_insn_seq_recursive<V: LirVisitor>(
        visitor: &mut V,
        function: &Function,
        seq_id: InsnSeqId,
        // next_val: &mut ValRef,
    ) {
        // let seq_id = *curr_seq_id;
        visitor.start_insn_seq(seq_id);
        for (idx, insn) in function.insn_seqs[seq_id].iter_enumerated() {
            let pc = InsnPc(seq_id, idx);
            visitor.visit_insn(insn, pc);
            // next_val.0 += insn.num_outputs();
            match insn {
                InsnKind::Block(block) => {
                    visit_insn_seq_recursive(visitor, function, block.body /* next_val */);
                }
                InsnKind::IfElse(if_else) => {
                    visit_insn_seq_recursive(
                        visitor,
                        function,
                        if_else.then_body, /* next_val */
                    );
                    visit_insn_seq_recursive(
                        visitor,
                        function,
                        if_else.else_body, /* next_val */
                    );
                }
                InsnKind::Loop(r#loop) => {
                    visit_insn_seq_recursive(visitor, function, r#loop.body /* next_val */);
                }
                _ => {}
            }
        }
        visitor.end_insn_seq(seq_id);
    }
}

pub fn infer_output_types(function: &Function) -> HashMap<ValRef, ValType> {
    struct InferOutputTypesVisitor {
        types: HashMap<ValRef, ValType>,
    }
    impl LirVisitor for InferOutputTypesVisitor {
        fn start_insn_seq(&mut self, _id: InsnSeqId) {}

        fn end_insn_seq(&mut self, _id: InsnSeqId) {}

        fn visit_insn(&mut self, insn: &InsnKind, pc: InsnPc) {
            // make sure that we are assigning the correct val ref
            // assert_eq!(next_val_ref, self.types.next_key());

            match insn {
                InsnKind::Const { r#type, .. } => {
                    self.types.insert(ValRef(pc, 0), *r#type);
                }
                InsnKind::DeriveField { .. } => {
                    self.types.insert(ValRef(pc, 0), ValType::Ptr);
                }
                InsnKind::DeriveElement { .. } => {
                    self.types.insert(ValRef(pc, 0), ValType::Ptr);
                }
                InsnKind::MemLoad { r#type, .. } => {
                    self.types.insert(ValRef(pc, 0), *r#type);
                }
                InsnKind::MemStore { .. } => {}
                InsnKind::StackLoad { r#type, .. } => {
                    self.types.insert(ValRef(pc, 0), *r#type);
                }
                InsnKind::StackStore { .. } => {}
                InsnKind::UnaryOp { op, operand } => {
                    self.types.insert(
                        ValRef(pc, 0),
                        infer_unary_op_output_type(*op, self.types[operand]),
                    );
                }
                InsnKind::BinaryOp { op, lhs, rhs } => {
                    self.types.insert(
                        ValRef(pc, 0),
                        infer_binary_op_output_type(*op, self.types[lhs], self.types[rhs]),
                    );
                }
                InsnKind::Break { .. } => {}
                InsnKind::ConditionalBreak { .. } => {}
                InsnKind::FunctionArgs { output_type, .. }
                | InsnKind::Block(Block { output_type, .. })
                | InsnKind::IfElse(IfElse { output_type, .. })
                | InsnKind::Loop(Loop { output_type, .. })
                | InsnKind::CallImportedFunction { output_type, .. }
                | InsnKind::CallUserFunction { output_type, .. } => {
                    for (idx, ty) in output_type.as_ref().iter().enumerate() {
                        self.types.insert(ValRef(pc, idx.try_into().unwrap()), *ty);
                    }
                }
                InsnKind::LoopArg { initial_value } => {
                    self.types.insert(ValRef(pc, 0), self.types[initial_value]);
                }
            };
        }
    }

    let mut visitor = InferOutputTypesVisitor { types: HashMap::new() };
    visit_insn_seq(&mut visitor, function);
    visitor.types
}

fn infer_unary_op_output_type(op: UnaryOpcode, operand: ValType) -> ValType {
    use UnaryOpcode::*;
    use ValType::*;

    match (op, operand) {
        (I64ToI32, I64) => I32,
        _ => todo!(),
    }
}

fn infer_binary_op_output_type(op: BinaryOpcode, lhs: ValType, rhs: ValType) -> ValType {
    use BinaryOpcode::*;
    use ValType::*;

    match (op, lhs, rhs) {
        (Add, I32, I32) => I32,
        _ => todo!(),
    }
}
