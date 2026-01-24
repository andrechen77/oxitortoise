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

use std::{collections::HashMap, fmt::Debug, rc::Rc};

use derive_more::{Deref, Display, From, Into};
use slotmap::{SecondaryMap, new_key_type};
use smallvec::SmallVec;
use typed_index_collections::TiVec;

mod boilerplate_impls;
mod macros;

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Program {
    pub entrypoints: Vec<FunctionId>,
    pub user_functions: SecondaryMap<FunctionId, Function>,
}

new_key_type! {
    pub struct FunctionId;
}

#[derive(Debug, Clone)]
pub struct HostFunctionInfo {
    pub parameter_types: &'static [ValType],
    pub return_type: &'static [ValType],
    /// Meaningful on Wasm targets only. Import the function by this name.
    pub name: &'static str,
}

#[derive(Debug, Copy, Clone, Deref)]
#[deref(forward)]
pub struct HostFunction(pub &'static HostFunctionInfo);

impl PartialEq for HostFunction {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}
impl Eq for HostFunction {}

impl std::hash::Hash for HostFunction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0 as *const HostFunctionInfo).hash(state);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Function {
    /// The non-stack local variables in this function. These is distinct from
    /// bytes stored on the stack. Function arguments are stored in the local
    /// variables upon entering the function.
    pub local_vars: TiVec<VarId, ValType>,
    /// The first `num_parameters` local variables are the function arguments.
    pub num_parameters: usize,
    /// The number of bytes to allocate on the stack for this function.
    pub stack_space: usize,
    pub body: Block,
    pub insn_seqs: TiVec<InsnSeqId, TiVec<InsnIdx, InsnKind>>,
    pub debug_fn_name: Option<Rc<str>>,
    pub debug_val_names: HashMap<ValRef, Rc<str>>,
    pub debug_var_names: HashMap<VarId, Rc<str>>,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, From, Into, Debug, Display)]
pub struct InsnSeqId(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Into, From, Debug, Display)]
pub struct InsnIdx(pub usize);

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Display)]
#[display("{_0}:{_1}")]
pub struct InsnPc(pub InsnSeqId, pub InsnIdx);

/// A reference to a value produced by an instruction. Starts from 0 and counts
/// up for each value produced by an instruction in the function. Some
/// instructions may produce multiple values, while others may produce zero.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Display)]
#[display("{_0}:{_1}")]
pub struct ValRef(pub InsnPc, pub u8);

#[derive(PartialEq, Eq, Hash, Clone, Copy, From, Into, Debug, Display)]
#[display("{self:?}")]
pub struct VarId(pub usize);

#[derive(PartialEq, Eq, Debug, Display)]
pub enum InsnKind {
    // QUESTION rethink loop args.
    // first, they should output a multivalue, instead of being a single value,
    // since that's what it will look like on the stack machine. In addition,
    // should the loop args instruction be considered to output values on the
    // stack, or does the instruction only exist so that other instructions can
    // have a ValRef to use the loop args? If the latter, then I need to modify
    // the stackify_lir code so that entering a loop body *doesn't* start with a
    // fresh stack. If the former, then I'll have to deal with the fact that
    // every loop body must start with a loop args instruction.
    /// Outputs all arguments of a loop body.
    #[display("loop_args(-> {})", initial_value)]
    LoopArg {
        /// The initial value of the arguments when the loop is entered.
        /// This is not considered an input to this instruction.
        initial_value: ValRef,
    },
    #[display("constant(ty={}, value={})", _0.ty, _0.value)]
    Const(Const),
    #[display("user_fn_ptr({:?})", function)]
    UserFunctionPtr {
        function: FunctionId,
    },
    /// Add a compile-time offset to a pointer, producing a pointer to a
    /// subfield.
    #[display("derive_field(offset={})({})", offset, ptr)]
    DeriveField {
        offset: usize,
        ptr: ValRef,
    },
    /// Add a dynamic offset to a pointer, producing a pointer to an element of
    /// an array.
    #[display("derive_element(stride={})(ptr={}, index={})", element_size, ptr, index)]
    DeriveElement {
        element_size: usize,
        ptr: ValRef,
        index: ValRef,
    },
    /// Load a value from memory.
    ///
    /// For loading values from the current function's stack frame, use [`InstructionKind::StackLoad`]
    #[display("mem_load(ty={}, offset={})({})", r#type, offset, ptr)]
    MemLoad {
        r#type: MemOpType,
        offset: usize,
        ptr: ValRef,
    },
    /// Store a value into memory.
    ///
    /// For storing values onto the current function's stack frame, use [`InstructionKind::StackStore`]
    #[display("mem_store(offset={})(ptr={}, value={})", offset, ptr, value)]
    MemStore {
        r#type: MemOpType,
        offset: usize,
        ptr: ValRef,
        value: ValRef,
    },
    /// Load a value from the stack.
    #[display("stack_load(ty={}, offset={})", r#type, offset)]
    StackLoad {
        r#type: MemOpType,
        /// The offset from the top of the stack at which to load the value.
        offset: usize,
    },
    /// Store a value onto the stack.
    #[display("stack_store(offset={})(value={})", offset, value)]
    StackStore {
        r#type: MemOpType,
        /// The offset from the top of the stack at which to store the value.
        offset: usize,
        value: ValRef,
    },
    /// Store a value into a possibly mutable local variable.
    #[display("var_store({})(value={})", var_id, value)]
    VarStore {
        var_id: VarId,
        value: ValRef,
    },
    /// Load a value from a possibly mutable local variable.
    #[display("var_load({})", var_id)]
    VarLoad {
        var_id: VarId,
    },
    /// Get the address of a value on the stack
    #[display("stack_addr({})", offset)]
    StackAddr {
        /// The offset from the top of the stack.
        offset: usize,
    },
    #[display("call_host_fn({:?}, -> {:?})({:?})", function, output_type, args)]
    CallHostFunction {
        function: HostFunction,
        output_type: SmallVec<[ValType; 1]>,
        args: Box<[ValRef]>,
    },
    #[display("call_user_fn({:?}, -> {:?})({:?})", function, output_type, args)]
    CallUserFunction {
        function: FunctionId,
        output_type: SmallVec<[ValType; 1]>,
        args: Box<[ValRef]>,
    },
    #[display("call_indirect_fn({:?}, -> {:?})({:?})", function, output_type, args)]
    CallIndirectFunction {
        function: ValRef,
        output_type: SmallVec<[ValType; 1]>,
        args: Box<[ValRef]>,
    },
    #[display("unary_op({})({})", op, operand)]
    UnaryOp {
        op: UnaryOpcode,
        operand: ValRef,
    },
    #[display("binary_op({})({}, {})", op, lhs, rhs)]
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
    #[display("break({})({:?})", target, values)]
    Break {
        target: InsnSeqId,
        values: Box<[ValRef]>,
    },
    /// Conditionally break out of control flow constructs. See
    /// [`InsnKind::Break`] for more information.
    #[display("conditional_break({})(cond={}, {:?})", target, condition, values)]
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

#[derive(PartialEq, Eq, Debug)]
pub struct Const {
    pub ty: ValType,
    /// The bit pattern of the value to store.
    pub value: u64,
}

impl Const {
    pub const NULL: Self = Self { ty: ValType::Ptr, value: 0 };
}

#[derive(PartialEq, Eq, Debug, Display)]
#[display("block(-> {:?}) {{{}}}", output_type, body)]
pub struct Block {
    /// The type of output of this block.
    pub output_type: SmallVec<[ValType; 1]>,
    /// The instructions inside the block.
    pub body: InsnSeqId,
}

#[derive(PartialEq, Eq, Debug, Display)]
#[display("if_else(-> {:?})({}) {{{}}} {{{}}}", output_type, condition, then_body, else_body)]
pub struct IfElse {
    /// The type of output of this if-else.
    pub output_type: SmallVec<[ValType; 1]>,
    /// The condition of the if-else.
    pub condition: ValRef,
    /// The instructions inside the then branch.
    pub then_body: InsnSeqId,
    /// The instructions inside the else branch.
    pub else_body: InsnSeqId,
}

#[derive(PartialEq, Eq, Debug, Display)]
#[display("loop(-> {:?}) {{{}}}", output_type, body)]
pub struct Loop {
    /// The initial values of the loop body arguments.
    pub inputs: Vec<ValRef>,
    /// The type of output of this loop.
    pub output_type: SmallVec<[ValType; 1]>,
    /// The instructions inside the loop.
    pub body: InsnSeqId,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Display)]
pub enum UnaryOpcode {
    FNeg,
    Not,
    I64ToI32,
}

/// The binary operations that can be performed. Variants are named according to
/// a certain prefixing convention.
/// - `I` operations apply to integer types.
/// - `S` operations apply to signed integer types.
/// - `U` operations apply to unsigned integer types.
/// - `F` operations apply to floating point types.
///
/// Pointers are considered unsigned integers at the LIR level.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Display)]
pub enum BinaryOpcode {
    IAdd,
    ISub,
    IMul,
    SLt,
    SGt,
    ULt,
    UGt,
    IEq,
    INeq,
    FAdd,
    FSub,
    FMul,
    FDiv,
    FLt,
    FLte,
    FGt,
    FGte,
    FEq,
    And,
    Or,
}

/// A machine-level type. These are just numbers that have no higher-level
/// semantic meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Display)]
pub enum ValType {
    I32,
    I64,
    F64,
    Ptr,
    FnPtr,
}

/// The type to use when loading from or storing to memory.
#[derive(PartialEq, Eq, Debug, Display, Clone, Copy)]
pub enum MemOpType {
    I8,
    I32,
    I64,
    F64,
    Ptr,
    FnPtr,
}

impl MemOpType {
    /// The type that a register value loaded with this mem op will have.
    pub fn loaded_type(&self) -> ValType {
        match self {
            Self::I8 => ValType::I32,
            Self::I32 => ValType::I32,
            Self::I64 => ValType::I64,
            Self::F64 => ValType::F64,
            Self::Ptr => ValType::Ptr,
            Self::FnPtr => ValType::FnPtr,
        }
    }
}

pub trait LirVisitor {
    fn start_insn_seq(&mut self, id: InsnSeqId) {
        let _ = id;
    }

    fn end_insn_seq(&mut self, id: InsnSeqId) {
        let _ = id;
    }

    /// Visits an instruction. `next_val_ref` is the index of the first output
    /// of this instruction, if it outputs any; this is used for assertions to
    /// ensure that output counting was correct.
    fn visit_insn(&mut self, insn: &InsnKind, pc: InsnPc) {
        let _ = insn;
        let _ = pc;
    }
}

pub fn visit_insn_seq<V: LirVisitor>(visitor: &mut V, function: &Function) {
    visit_insn_seq_recursive(visitor, function, function.body.body);
    fn visit_insn_seq_recursive<V: LirVisitor>(
        visitor: &mut V,
        function: &Function,
        seq_id: InsnSeqId,
    ) {
        visitor.start_insn_seq(seq_id);
        for (idx, insn) in function.insn_seqs[seq_id].iter_enumerated() {
            let pc = InsnPc(seq_id, idx);
            visitor.visit_insn(insn, pc);
            match insn {
                InsnKind::Block(block) => {
                    visit_insn_seq_recursive(visitor, function, block.body);
                }
                InsnKind::IfElse(if_else) => {
                    visit_insn_seq_recursive(visitor, function, if_else.then_body);
                    visit_insn_seq_recursive(visitor, function, if_else.else_body);
                }
                InsnKind::Loop(r#loop) => {
                    visit_insn_seq_recursive(visitor, function, r#loop.body);
                }
                _ => {}
            }
        }
        visitor.end_insn_seq(seq_id);
    }
}

pub fn infer_output_types(function: &Function) -> HashMap<ValRef, ValType> {
    struct InferOutputTypesVisitor<'a> {
        types: HashMap<ValRef, ValType>,
        var_types: &'a TiVec<VarId, ValType>,
    }
    impl<'a> LirVisitor for InferOutputTypesVisitor<'a> {
        fn start_insn_seq(&mut self, _id: InsnSeqId) {}

        fn end_insn_seq(&mut self, _id: InsnSeqId) {}

        fn visit_insn(&mut self, insn: &InsnKind, pc: InsnPc) {
            match insn {
                InsnKind::Const(Const { ty: r#type, .. }) => {
                    self.types.insert(ValRef(pc, 0), *r#type);
                }
                InsnKind::UserFunctionPtr { .. } => {
                    self.types.insert(ValRef(pc, 0), ValType::FnPtr);
                }
                InsnKind::DeriveField { .. } => {
                    self.types.insert(ValRef(pc, 0), ValType::Ptr);
                }
                InsnKind::DeriveElement { .. } => {
                    self.types.insert(ValRef(pc, 0), ValType::Ptr);
                }
                InsnKind::MemLoad { r#type, .. } => {
                    self.types.insert(ValRef(pc, 0), r#type.loaded_type());
                }
                InsnKind::MemStore { .. } => {}
                InsnKind::StackLoad { r#type, .. } => {
                    self.types.insert(ValRef(pc, 0), r#type.loaded_type());
                }
                InsnKind::StackStore { .. } => {}
                InsnKind::StackAddr { .. } => {
                    self.types.insert(ValRef(pc, 0), ValType::Ptr);
                }
                InsnKind::VarStore { .. } => {}
                InsnKind::VarLoad { var_id } => {
                    self.types.insert(ValRef(pc, 0), self.var_types[*var_id]);
                }
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
                InsnKind::Block(Block { output_type, .. })
                | InsnKind::IfElse(IfElse { output_type, .. })
                | InsnKind::Loop(Loop { output_type, .. })
                | InsnKind::CallHostFunction { output_type, .. }
                | InsnKind::CallUserFunction { output_type, .. }
                | InsnKind::CallIndirectFunction { output_type, .. } => {
                    for (idx, ty) in output_type.iter().enumerate() {
                        self.types.insert(ValRef(pc, idx.try_into().unwrap()), *ty);
                    }
                }
                InsnKind::LoopArg { initial_value } => {
                    self.types.insert(ValRef(pc, 0), self.types[initial_value]);
                }
            };
        }
    }

    let mut visitor =
        InferOutputTypesVisitor { types: HashMap::new(), var_types: &function.local_vars };
    visit_insn_seq(&mut visitor, function);
    visitor.types
}

fn infer_unary_op_output_type(op: UnaryOpcode, operand: ValType) -> ValType {
    use UnaryOpcode as O;
    use ValType as V;

    match (op, operand) {
        (O::I64ToI32, V::I64) => V::I32,
        (O::FNeg, V::F64) => V::F64,
        (O::Not, V::I32) => V::I32,
        _ => todo!(
            "TODO(mvp) add other compinations of ops and val types {:?} and {:?}",
            op,
            operand
        ),
    }
}

fn infer_binary_op_output_type(op: BinaryOpcode, lhs: ValType, rhs: ValType) -> ValType {
    use BinaryOpcode as B;
    use ValType as V;

    match op {
        B::IAdd | B::ISub | B::IMul => match (lhs, rhs) {
            (V::I32, V::I32) => V::I32,
            (V::I64, V::I64) => V::I64,
            (V::Ptr, V::Ptr) => V::Ptr,
            _ => panic!("Invalid operand types for operation {:?}: {:?} and {:?}", op, lhs, rhs),
        },
        B::ULt | B::UGt | B::IEq | B::INeq => match (lhs, rhs) {
            (V::I32, V::I32) => V::I32,
            (V::I64, V::I64) => V::I32,
            (V::Ptr, V::Ptr) => V::I32,
            _ => panic!("Invalid operand types for operation {:?}: {:?} and {:?}", op, lhs, rhs),
        },
        B::SLt | B::SGt => match (lhs, rhs) {
            (V::I32, V::I32) => V::I32,
            (V::I64, V::I64) => V::I32,
            _ => panic!("Invalid operand types for operation {:?}: {:?} and {:?}", op, lhs, rhs),
        },
        B::FAdd | B::FSub | B::FMul | B::FDiv => match (lhs, rhs) {
            (V::F64, V::F64) => V::F64,
            _ => panic!("Invalid operand types for operation {:?}: {:?} and {:?}", op, lhs, rhs),
        },
        B::FLt | B::FLte | B::FGt | B::FGte | B::FEq => match (lhs, rhs) {
            (V::F64, V::F64) => V::I32,
            _ => panic!("Invalid operand types for operation {:?}: {:?} and {:?}", op, lhs, rhs),
        },
        B::And | B::Or => match (lhs, rhs) {
            (V::I32, V::I32) => V::I32,
            _ => panic!("Invalid operand types for operation {:?}: {:?} and {:?}", op, lhs, rhs),
        },
    }
}

pub fn generate_host_function_call(function: HostFunction, args: Box<[ValRef]>) -> InsnKind {
    // TODO(mvp) validate that the types and number of arguments match
    InsnKind::CallHostFunction { function, output_type: function.return_type.into(), args }
}

pub fn host_function_references(program: &Program) -> Vec<HostFunction> {
    struct HostFnCollector {
        host_fns: Vec<HostFunction>,
    }

    impl LirVisitor for HostFnCollector {
        fn visit_insn(&mut self, insn: &InsnKind, _pc: InsnPc) {
            if let InsnKind::CallHostFunction { function, .. } = insn {
                self.host_fns.push(*function);
            }
        }
    }

    let mut collector = HostFnCollector { host_fns: vec![] };
    for function in program.user_functions.values() {
        visit_insn_seq(&mut collector, function);
    }
    collector.host_fns
}
