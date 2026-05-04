//! Low-level intermediate representation, intended to be converted to
//! platform-specific code (e.g. Wasm, CLIF).
//!
//! # Instructions
//!
//! Each instruction in a function takes some number of inputs and has some
//! number of outputs. The outputs of instructions are assigned to mutable local
//! variables referred to as "registers". Unlike actual physical architectures,
//! there are an infinite number of registers used by the program, and reusing
//! registers to "save space" is not required. Functions can use the current
//! values of registers as inputs. The function body is a single instruction
//! sequence, but instruction sequences can "contain" other instruction
//! sequences through compound instructions. Control flow is structured (rather
//! than a control flow graph), being handled by special instructions for
//! blocks, if-else, and loops.
//!
//! Instructions are identified by an index within their containing sequence,
//! this is their [`InsnIdx`]. Instruction sequences also have their own unique
//! ID, [`InsnSeqId`]. An instruction sequence ID and an index together uniquely
//! identify an instruction, known as the [`InsnPc`].
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

use std::{alloc::Layout, fmt::Debug, sync::Arc};

use derive_more::{Deref, Display, From, Into};
use slotmap::{SecondaryMap, new_key_type};
use smallvec::SmallVec;
use typed_index_collections::TiVec;

mod boilerplate_impls;

#[derive(PartialEq, Eq, Default)]
pub struct Program {
    pub user_functions: SecondaryMap<FunctionId, Function>,
}

new_key_type! {
    pub struct FunctionId;
}

#[derive(Debug, Clone)]
pub struct HostFunctionInfo {
    pub parameter_types: &'static [ValType],
    pub return_type: &'static [ValType],
    pub addr: *const u8,
    /// Meaningful on Wasm targets only. Import the function by this name.
    pub name: &'static str,
}

/// The only thing that is not recognized as safe to share is the addr, but that
/// is always used for a function pointer so it is actually safe to share.
unsafe impl Sync for HostFunctionInfo {}

#[derive(Debug, Copy, Clone, Deref, Display)]
#[deref(forward)]
#[display("{:?}", self.0.name)]
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

#[derive(PartialEq, Eq)]
pub struct Function {
    /// The non-stack local variables in this function. These is distinct from
    /// bytes stored on the stack. Function arguments are stored in the local
    /// variables upon entering the function.
    pub registers: TiVec<Reg, RegInfo>,
    pub parameters: Vec<Reg>,
    pub return_values: Vec<Reg>,
    /// The number of bytes to allocate on the stack for this function.
    pub stack_space: usize,
    pub body: Block,
    pub insn_seqs: TiVec<InsnSeqId, Vec<InsnKind>>,
    pub debug_fn_name: Option<Arc<str>>,
    pub is_entrypoint: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RegInfo {
    pub ty: ValType,
    pub name: Option<Arc<str>>,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, From, Into, Debug)]
pub struct InsnSeqId(pub usize);

#[derive(PartialEq, Eq, Hash, Clone, Copy, From, Into)]
pub struct Reg(pub usize);

#[derive(PartialEq, Eq, Clone)]
pub enum InsnKind {
    /// Produces a single value which is stored in the specified register.
    SingleVal {
        out: Reg,
        insn: SingleValInsn,
    },
    /// Produces a multiple values which are stored in the specified registers.
    MultiVal {
        out: SmallVec<[Reg; 1]>,
        insn: MultiValInsn,
    },
    /// Store a value into memory.
    ///
    /// For storing values onto the current function's stack frame, use [`InstructionKind::StackStore`]
    MemStore {
        r#type: ValType,
        offset: usize,
        ptr: Reg,
        value: Reg,
    },
    /// Store a value onto the stack.
    StackStore {
        r#type: ValType,
        /// The offset from the top of the stack at which to store the value.
        offset: usize,
        value: Reg,
    },
    /// Break out of some number of control flow constructs. This must be the
    /// last instruction within an instruction sequence.
    ///
    /// # Target
    ///
    /// The target parameter is the [`InsnSeqId`] of the sequence to break out
    /// of. The continuation depends on the compound instruction that the
    /// instruction sequence belongs to. If it belongs to a block or if-else,
    /// then this will jump to the end of the block/if-else. Ff it belongs to a
    /// loop, then this will jump to the start of the loop body. Since
    /// `InsnSeqId(0)` is always the entire function, that target can be used to
    /// implement early returns.
    Break {
        target: InsnSeqId,
    },
    /// Conditionally break out of control flow constructs. See
    /// [`InsnKind::Break`] for more information.
    ConditionalBreak {
        target: InsnSeqId,
        condition: Reg,
    },
    // TODO add a fallthrough instruction to allow returning values from a
    // loop without wrapping to the top of the loop again.
    Block(Block),
    IfElse(IfElse),
    Loop(Loop),
}

#[derive(PartialEq, Eq, Clone)]
pub enum SingleValInsn {
    Const {
        val: Value,
    },
    UserFunctionPtr {
        function: FunctionId,
    },
    /// Add a compile-time offset to a pointer, producing a pointer to a
    /// subfield.
    DeriveField {
        offset: usize,
        ptr: Reg,
    },
    /// Add a dynamic offset to a pointer, producing a pointer to an element of
    /// an array.
    DeriveElement {
        element_size: usize,
        ptr: Reg,
        index: Reg,
    },
    /// Load a value from memory.
    ///
    /// For loading values from the current function's stack frame, use [`InstructionKind::StackLoad`]
    MemLoad {
        r#type: ValType,
        offset: usize,
        ptr: Reg,
    },
    /// Load a value from the stack.
    StackLoad {
        r#type: ValType,
        /// The offset from the top of the stack at which to load the value.
        offset: usize,
    },
    /// Get the address of a value on the stack
    StackAddr {
        /// The offset from the top of the stack.
        offset: usize,
    },
    UnaryOp {
        op: UnaryOpcode,
        operand: Reg,
    },
    BinaryOp {
        /// The operation to perform. This also determines the types of the
        /// inputs and outputs.
        op: BinaryOpcode,
        lhs: Reg,
        rhs: Reg,
    },
}

#[derive(PartialEq, Eq, Clone)]
pub enum MultiValInsn {
    CallHostFunction { function: HostFunction, args: Box<[Reg]> },
    CallUserFunction { function: FunctionId, args: Box<[Reg]> },
    CallIndirectFunction { function: Reg, args: Box<[Reg]> },
}

#[derive(Debug, Clone, Copy)]
pub enum Value {
    I8(u8),
    I32(u32),
    I64(u64),
    F64(f64),
    Ptr(*const u8),
    FnPtr(*const u8),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;
        match (self, other) {
            (I32(a), I32(b)) => a == b,
            (I64(a), I64(b)) => a == b,
            (F64(a), F64(b)) => a.to_bits() == b.to_bits(),
            (Ptr(a), Ptr(b)) => a.addr() == b.addr(),
            (FnPtr(a), FnPtr(b)) => a.addr() == b.addr(),
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Value {
    pub const NULL: Self = Self::Ptr(std::ptr::null());

    fn ty(&self) -> ValType {
        match self {
            Self::I8(_) => ValType::I8,
            Self::I32(_) => ValType::I32,
            Self::I64(_) => ValType::I64,
            Self::F64(_) => ValType::F64,
            Self::Ptr(_) => ValType::Ptr,
            Self::FnPtr(_) => ValType::FnPtr,
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct Block {
    /// The instructions inside the block.
    pub body: InsnSeqId,
}

#[derive(PartialEq, Eq, Clone)]
pub struct IfElse {
    /// The condition of the if-else.
    pub condition: Reg,
    /// The instructions inside the then branch.
    pub then_body: InsnSeqId,
    /// The instructions inside the else branch.
    pub else_body: InsnSeqId,
}

#[derive(PartialEq, Eq, Clone)]
pub struct Loop {
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
    I8,
    I32,
    I64,
    F64,
    Ptr,
    FnPtr,
}

impl ValType {
    pub fn layout(&self) -> Layout {
        match self {
            Self::I8 => Layout::new::<u8>(),
            Self::I32 => Layout::new::<u32>(),
            Self::I64 => Layout::new::<u64>(),
            Self::F64 => Layout::new::<f64>(),
            Self::Ptr => Layout::new::<*const u8>(),
            Self::FnPtr => Layout::new::<*const u8>(),
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
    fn visit_insn(&mut self, insn: &InsnKind) {
        let _ = insn;
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
        for insn in &function.insn_seqs[seq_id] {
            visitor.visit_insn(insn);
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

impl SingleValInsn {
    pub fn output_type(&self, input_regs: &TiVec<Reg, RegInfo>) -> ValType {
        match self {
            SingleValInsn::Const { val } => val.ty(),
            SingleValInsn::UserFunctionPtr { .. } => ValType::FnPtr,
            SingleValInsn::DeriveField { .. } => ValType::Ptr,
            SingleValInsn::DeriveElement { .. } => ValType::Ptr,
            SingleValInsn::MemLoad { r#type, .. } => *r#type,
            SingleValInsn::StackLoad { r#type, .. } => *r#type,
            SingleValInsn::StackAddr { .. } => ValType::Ptr,
            SingleValInsn::UnaryOp { op, operand } => {
                infer_unary_op_output_type(*op, input_regs[*operand].ty)
            }
            SingleValInsn::BinaryOp { op, lhs, rhs } => {
                infer_binary_op_output_type(*op, input_regs[*lhs].ty, input_regs[*rhs].ty)
            }
        }
    }
}

// pub fn infer_output_types(function: &Function) -> HashMap<ValRef, ValType> {
//     struct InferOutputTypesVisitor<'a> {
//         types: HashMap<ValRef, ValType>,
//         var_types: &'a TiVec<VarId, ValType>,
//     }
//     impl<'a> LirVisitor for InferOutputTypesVisitor<'a> {
//         fn start_insn_seq(&mut self, _id: InsnSeqId) {}

//         fn end_insn_seq(&mut self, _id: InsnSeqId) {}

//         fn visit_insn(&mut self, insn: &InsnKind, pc: InsnPc) {
//             match insn {
//                 InsnKind::Const(value) => {
//                     self.types.insert(ValRef(pc, 0), value.ty());
//                 }
//                 InsnKind::UserFunctionPtr { .. } => {
//                     self.types.insert(ValRef(pc, 0), ValType::FnPtr);
//                 }
//                 InsnKind::DeriveField { .. } => {
//                     self.types.insert(ValRef(pc, 0), ValType::Ptr);
//                 }
//                 InsnKind::DeriveElement { .. } => {
//                     self.types.insert(ValRef(pc, 0), ValType::Ptr);
//                 }
//                 InsnKind::MemLoad { r#type, .. } => {
//                     self.types.insert(ValRef(pc, 0), *r#type);
//                 }
//                 InsnKind::MemStore { .. } => {}
//                 InsnKind::StackLoad { r#type, .. } => {
//                     self.types.insert(ValRef(pc, 0), *r#type);
//                 }
//                 InsnKind::StackStore { .. } => {}
//                 InsnKind::StackAddr { .. } => {
//                     self.types.insert(ValRef(pc, 0), ValType::Ptr);
//                 }
//                 InsnKind::VarStore { .. } => {}
//                 InsnKind::VarLoad { var_id } => {
//                     self.types.insert(ValRef(pc, 0), self.var_types[*var_id]);
//                 }
//                 InsnKind::UnaryOp { op, operand } => {
//                     self.types.insert(
//                         ValRef(pc, 0),
//                         infer_unary_op_output_type(*op, self.types[operand]),
//                     );
//                 }
//                 InsnKind::BinaryOp { op, lhs, rhs } => {
//                     self.types.insert(
//                         ValRef(pc, 0),
//                         infer_binary_op_output_type(*op, self.types[lhs], self.types[rhs]),
//                     );
//                 }
//                 InsnKind::Break { .. } => {}
//                 InsnKind::ConditionalBreak { .. } => {}
//                 InsnKind::Block(Block { output_type, .. })
//                 | InsnKind::IfElse(IfElse { output_type, .. })
//                 | InsnKind::Loop(Loop { output_type, .. })
//                 | InsnKind::CallHostFunction { output_type, .. }
//                 | InsnKind::CallUserFunction { output_type, .. }
//                 | InsnKind::CallIndirectFunction { output_type, .. } => {
//                     for (idx, ty) in output_type.iter().enumerate() {
//                         self.types.insert(ValRef(pc, idx.try_into().unwrap()), *ty);
//                     }
//                 }
//                 InsnKind::LoopArg { initial_value } => {
//                     self.types.insert(ValRef(pc, 0), self.types[initial_value]);
//                 }
//             };
//         }
//     }

//     let mut visitor =
//         InferOutputTypesVisitor { types: HashMap::new(), var_types: &function.registers };
//     visit_insn_seq(&mut visitor, function);
//     visitor.types
// }

fn infer_unary_op_output_type(op: UnaryOpcode, operand: ValType) -> ValType {
    use UnaryOpcode as O;
    use ValType as V;

    match (op, operand) {
        (O::I64ToI32, V::I64) => V::I32,
        (O::FNeg, V::F64) => V::F64,
        (O::Not, V::I8) => V::I8,
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
            (V::I32, V::I32) => V::I8,
            (V::I64, V::I64) => V::I8,
            (V::Ptr, V::Ptr) => V::I8,
            _ => panic!("Invalid operand types for operation {:?}: {:?} and {:?}", op, lhs, rhs),
        },
        B::SLt | B::SGt => match (lhs, rhs) {
            (V::I32, V::I32) => V::I8,
            (V::I64, V::I64) => V::I8,
            _ => panic!("Invalid operand types for operation {:?}: {:?} and {:?}", op, lhs, rhs),
        },
        B::FAdd | B::FSub | B::FMul | B::FDiv => match (lhs, rhs) {
            (V::F64, V::F64) => V::F64,
            _ => panic!("Invalid operand types for operation {:?}: {:?} and {:?}", op, lhs, rhs),
        },
        B::FLt | B::FLte | B::FGt | B::FGte | B::FEq => match (lhs, rhs) {
            (V::F64, V::F64) => V::I8,
            _ => panic!("Invalid operand types for operation {:?}: {:?} and {:?}", op, lhs, rhs),
        },
        B::And | B::Or => match (lhs, rhs) {
            (V::I8, V::I8) => V::I8,
            _ => panic!("Invalid operand types for operation {:?}: {:?} and {:?}", op, lhs, rhs),
        },
    }
}

// pub fn host_function_references(program: &Program) -> HashSet<HostFunction> {
//     struct HostFnCollector {
//         host_fns: HashSet<HostFunction>,
//     }

//     impl LirVisitor for HostFnCollector {
//         fn visit_insn(&mut self, insn: &InsnKind, _pc: InsnPc) {
//             if let InsnKind::CallHostFunction { function, .. } = insn {
//                 self.host_fns.insert(*function);
//             }
//         }
//     }

//     let mut collector = HostFnCollector { host_fns: HashSet::new() };
//     for function in program.user_functions.values() {
//         visit_insn_seq(&mut collector, function);
//     }
//     collector.host_fns
// }
