use std::{
    collections::{BTreeMap, HashMap},
    num::NonZero,
};

use lir::{smallvec::smallvec, typed_index_collections::ti_vec};
use tracing::{debug, info, trace};
use walrus::ir as wir;

use crate::stackify_generic::{self, StackManipulators};

struct CodegenModuleInfo<'a, A> {
    /// The id of the main memory of the module.
    mem_id: walrus::MemoryId,
    /// The id of the stack pointer global of the module.
    sp_global_id: walrus::GlobalId,
    /// The id of the main function table of the module.
    fn_table_id: walrus::TableId,
    /// An allocator that can be used to get new fn table slots.
    fn_table_slot_allocator: &'a mut A,
    /// A map from function ids to the slot in the function table that they
    /// (will) take up.
    fn_table_allocated_slots: HashMap<lir::FunctionId, usize>,
    /// A map from LIR function ids to Walrus function ids.
    user_fn_ids: HashMap<lir::FunctionId, walrus::FunctionId>,
    /// A map from LIR host functions to Walrus function ids.
    host_fn_ids: HashMap<lir::HostFunction, walrus::FunctionId>,
}

impl<'a, A> CodegenModuleInfo<'a, A> {
    fn lookup_host_fn(&self, host_fn: lir::HostFunction) -> walrus::FunctionId {
        debug!("looking up host function {:?}", host_fn);
        self.host_fn_ids[&host_fn]
    }
}

pub trait FnTableSlotAllocator {
    /// Return a free slot in the function table.
    fn allocate_slot(&mut self) -> usize;
}

/// Context for codegen that applies to the entire function.
struct CodegenFnCtx<'b> {
    /// The function whose code is being generated.
    func: &'b lir::Function,
    /// For each pair of a register and an insn seq:
    /// - The entry does not exist if the register is not live in that
    ///   instruction sequence.
    /// - Stores None if the register is live for the entire insn seq,
    /// - Stores the number of times that register is read in that instruction
    ///   sequence, where each child insn seq that reads the register at least
    ///   once counts as a single read for the parent insn seq.
    reg_liveness: &'b BTreeMap<(lir::InsnSeqId, lir::Reg), Option<NonZero<usize>>>,
    /// The walrus local variable that holds the stack pointer.
    sp_local_id: Option<wir::LocalId>,
    /// Maps each LIR register to the Wasm local variable that holds the value.
    reg_local_ids: BTreeMap<lir::Reg, wir::LocalId>,
    /// Associates each LIR insn seq id with the walrus insn seq id.
    seq_id_mapping: BTreeMap<lir::InsnSeqId, wir::InstrSeqId>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Val {
    Reg {
        /// The register whose value we want to use.
        reg: lir::Reg,
        /// A monotonically increasing index that differentiates between
        /// different values of the same register if the register was modified.
        nonce: u32,
    },
    /// The stack pointer of the current function. This cannot change.
    StackPtr,
}

#[derive(Default)]
struct ValTracker {
    /// Maps each register to the nonce of the last value of the register that
    /// was set. If a register does not exist in this map, its value was never
    /// set.
    current_reg_nonces: HashMap<lir::Reg, u32>,
}

impl ValTracker {
    fn set_reg(&mut self, reg: lir::Reg) -> Val {
        let nonce = self.current_reg_nonces.entry(reg).or_insert(0);
        *nonce += 1;
        Val::Reg { reg, nonce: *nonce }
    }

    fn get_reg(&self, reg: lir::Reg) -> Val {
        let nonce = self.current_reg_nonces.get(&reg).copied().unwrap_or(0);
        Val::Reg { reg, nonce }
    }
}

fn write_insn_seq<A: FnTableSlotAllocator>(
    module_locals: &mut walrus::ModuleLocals,
    module_types: &mut walrus::ModuleTypes,
    mod_info: &mut CodegenModuleInfo<A>,
    ctx: &mut CodegenFnCtx,
    insn_builder: &mut walrus::InstrSeqBuilder,
    insn_seq_id: lir::InsnSeqId,
) {
    let insn_seq = &ctx.func.insn_seqs[insn_seq_id];

    // let mut vals = ValTracker::default();

    let mut op_stack = Vec::new();
    let mut getters = HashMap::new();
    let mut manips = ti_vec![StackManipulators {
        captures: 0,
        getters: vec![],
    }; insn_seq.len() + 1];
    for (insn_idx, insn) in insn_seq.iter().enumerate() {
        let inputs;
        let outputs;
        match insn {
            lir::InsnKind::SingleVal { out, insn } => {
                match insn {
                    lir::SingleValInsn::Const { val } => {
                        inputs = smallvec![];
                        insn_builder.const_(translate_val(*val));
                    }
                    lir::SingleValInsn::UserFunctionPtr { function } => {
                        inputs = smallvec![];
                        let slot = addr_of_function(mod_info, *function);
                        insn_builder.const_(translate_usize(slot));
                    }
                    lir::SingleValInsn::DeriveField { offset, ptr } => {
                        inputs = smallvec![*ptr];
                        insn_builder.const_(translate_usize(*offset)).binop(wir::BinaryOp::I32Add);
                    }
                    lir::SingleValInsn::DeriveElement { element_size, ptr, index } => {
                        inputs = smallvec![*ptr, *index];
                        insn_builder
                            .const_(translate_usize(*element_size))
                            .binop(wir::BinaryOp::I32Mul)
                            .binop(wir::BinaryOp::I32Add);
                    }
                    lir::SingleValInsn::MemLoad { r#type, offset, ptr } => {
                        let load_kind = infer_load_kind(*r#type);
                        let mem_arg = infer_mem_arg(*r#type, *offset);

                        inputs = smallvec![*ptr];
                        insn_builder.load(mod_info.mem_id, load_kind, mem_arg);
                    }
                    lir::SingleValInsn::StackLoad { r#type, offset } => {
                        let load_kind = infer_load_kind(*r#type);
                        let mem_arg = infer_mem_arg(*r#type, *offset);
                        let stack_local = ctx.sp_local_id.expect("the presence of a StackLoad instruction means there must be a stack pointer local var");

                        inputs = smallvec![];
                        insn_builder.local_get(stack_local).load(
                            mod_info.mem_id,
                            load_kind,
                            mem_arg,
                        );
                    }
                    lir::SingleValInsn::StackAddr { offset } => {
                        let stack_local = ctx.sp_local_id.expect("the presence of a StackLoad instruction means there must be a stack pointer local var");

                        inputs = smallvec![];
                        insn_builder
                            .local_get(stack_local)
                            .const_(translate_usize(*offset))
                            .binop(wir::BinaryOp::I32Add);
                    }
                    lir::SingleValInsn::UnaryOp { op, operand } => {
                        inputs = smallvec![*operand];
                        insn_builder.unop(translate_unary_op(*op));
                    }
                    lir::SingleValInsn::BinaryOp { op, lhs, rhs } => {
                        inputs = smallvec![*lhs, *rhs];
                        insn_builder.binop(translate_binary_op(
                            *op,
                            ctx.func.registers[*lhs].ty,
                            ctx.func.registers[*rhs].ty,
                        ));
                    }
                };
                outputs = smallvec![*out];
            }
            lir::InsnKind::MultiVal { out, insn } => {
                match insn {
                    lir::MultiValInsn::CallHostFunction { function, args } => {
                        let callee = mod_info.lookup_host_fn(*function);

                        inputs = args.iter().copied().collect();
                        insn_builder.call(callee);
                    }
                    lir::MultiValInsn::CallUserFunction { function, args } => {
                        let callee = mod_info.user_fn_ids[function];

                        inputs = args.iter().copied().collect();
                        insn_builder.call(callee);
                    }
                    lir::MultiValInsn::CallIndirectFunction { function, args } => {
                        let fn_table_id = mod_info.fn_table_id;
                        let input_types: Vec<_> = args
                            .iter()
                            .map(|reg| translate_val_type(ctx.func.registers[*reg].ty))
                            .collect();
                        let output_types: Vec<_> = out
                            .iter()
                            .map(|reg| translate_val_type(ctx.func.registers[*reg].ty))
                            .collect();
                        let type_id = module_types.add(&input_types, &output_types);
                        let function_and_args = std::iter::once(function).chain(args.iter());

                        inputs = function_and_args.copied().collect();
                        insn_builder.call_indirect(type_id, fn_table_id);
                    }
                }
                outputs = out.iter().copied().collect();
            }
            lir::InsnKind::MemStore { r#type, offset, ptr, value } => {
                let store_kind = infer_store_kind(*r#type);
                let mem_arg = infer_mem_arg(*r#type, *offset);

                inputs = smallvec![*ptr, *value];
                insn_builder.store(mod_info.mem_id, store_kind, mem_arg);
                outputs = smallvec![];
            }
            lir::InsnKind::StackStore { r#type, offset, value } => {
                let store_kind = infer_store_kind(*r#type);
                let mem_arg = infer_mem_arg(*r#type, *offset);

                inputs = smallvec![Val::StackPtr, *value];
                insn_builder.store(mod_info.mem_id, store_kind, mem_arg);
                outputs = smallvec![];
            }
            lir::InsnKind::Break { target } => {
                let target = ctx.seq_id_mapping[target];

                inputs = smallvec![];
                insn_builder.br(target);
                outputs = smallvec![];
            }
            lir::InsnKind::ConditionalBreak { target, condition } => {
                let target = ctx.seq_id_mapping[target];

                inputs = smallvec![vals.get_reg(*condition)];
                insn_builder.br_if(target);
                outputs = smallvec![];
            }
            lir::InsnKind::Block(lir::Block { body }) => {
                let seq_type =
                    translate_instr_seq_type(module_types, std::iter::empty(), std::iter::empty());

                inputs = smallvec![];
                insn_builder.block(seq_type, |inner_builder| {
                    write_insn_seq(
                        module_locals,
                        module_types,
                        mod_info,
                        ctx,
                        inner_builder,
                        *body,
                    );
                });
                outputs = smallvec![];
            }
            lir::InsnKind::IfElse(lir::IfElse { condition, then_body, else_body }) => {
                let seq_type =
                    translate_instr_seq_type(module_types, std::iter::empty(), std::iter::empty());

                inputs = smallvec![vals.get_reg(*condition)];
                // interior mutability is needed because we need to borrow the
                // context twice in the two closures.
                let cell = std::cell::RefCell::new((
                    &mut *module_locals,
                    &mut *module_types,
                    &mut *mod_info,
                    &mut *ctx,
                ));
                insn_builder.if_else(
                    seq_type,
                    |then_builder| {
                        let (module_locals, module_types, mod_info, ctx) = &mut *cell.borrow_mut();
                        write_insn_seq(
                            module_locals,
                            module_types,
                            mod_info,
                            ctx,
                            then_builder,
                            *then_body,
                        );
                    },
                    |else_builder| {
                        let (module_locals, module_types, mod_info, ctx) = &mut *cell.borrow_mut();
                        write_insn_seq(
                            module_locals,
                            module_types,
                            mod_info,
                            ctx,
                            else_builder,
                            *else_body,
                        );
                    },
                );

                outputs = smallvec![];
            }
            lir::InsnKind::Loop(lir::Loop { body }) => {
                let seq_type =
                    translate_instr_seq_type(module_types, std::iter::empty(), std::iter::empty());

                inputs = smallvec![];
                insn_builder.loop_(seq_type, |inner_builder| {
                    write_insn_seq(
                        module_locals,
                        module_types,
                        mod_info,
                        ctx,
                        inner_builder,
                        *body,
                    );
                });
                outputs = smallvec![];
            }
        };

        // TODO for each output, make it so that any previous values of that
        // register left on the operand stack are invalidated.

        stackify_generic::stackify_single(
            &mut op_stack,
            &mut getters,
            &mut manips,
            insn_idx,
            inputs,
            outputs,
        );
    }

    // any excess operands on the stack should be eliminated
    stackify_generic::remove_excess_operands(
        op_stack.into_iter().zip(std::iter::repeat(false)),
        &mut manips,
        insn_seq.len(),
    );

    // TODO based on the current state of "manips", add manipulators to the
    // instruction sequence
    for (idx, manip) in manips.iter_enumerated() {
        let StackManipulators { captures, getters } = manip;
    }
}

fn addr_of_function<A: FnTableSlotAllocator>(
    mod_info: &mut CodegenModuleInfo<A>,
    function: lir::FunctionId,
) -> usize {
    use std::collections::hash_map::Entry;
    match mod_info.fn_table_allocated_slots.entry(function) {
        Entry::Occupied(entry) => *entry.get(),
        Entry::Vacant(entry) => {
            let new_slot = mod_info.fn_table_slot_allocator.allocate_slot();
            entry.insert(new_slot);
            new_slot
        }
    }
}

/// Translate a LIR value type to a Wasm value type.
fn translate_val_type(r#type: lir::ValType) -> walrus::ValType {
    match r#type {
        lir::ValType::I8 => walrus::ValType::I32, // there is no 8-bit register
        lir::ValType::I32 => walrus::ValType::I32,
        lir::ValType::I64 => walrus::ValType::I64,
        lir::ValType::F64 => walrus::ValType::F64,
        lir::ValType::Ptr => walrus::ValType::I32,
        lir::ValType::FnPtr => walrus::ValType::I32,
    }
}

/// Translate a LIR value to a Wasm value.
fn translate_val(value: lir::Value) -> walrus::ir::Value {
    match value {
        lir::Value::I8(value) => wir::Value::I32(value as i32),
        lir::Value::I32(value) => wir::Value::I32(value as i32),
        lir::Value::I64(value) => wir::Value::I64(value as i64),
        lir::Value::F64(value) => wir::Value::F64(value),
        // TODO this is only valid if pointers are 32 bits
        lir::Value::Ptr(value) => wir::Value::I32(value.addr() as i32),
        lir::Value::FnPtr(value) => wir::Value::I32(value.addr() as i32),
    }
}

/// Translates a pointer or pointer-sized value in LIR to a Wasm value. This
/// is preferred over using `as` casts because it type-checks that the input is
/// a `usize`, and picks the correct output type for the target platform.
// FIXME this function depends on the fact that the pointer width is 32 bits, so
// maybe annotate this with an attribute that fails compilation if this is not
// true?
fn translate_usize(x: usize) -> walrus::ir::Value {
    wir::Value::I32(x as i32)
}

fn infer_load_kind(r#type: lir::ValType) -> wir::LoadKind {
    use wir::ExtendedLoad as L;
    match r#type {
        lir::ValType::I8 => wir::LoadKind::I32_8 { kind: L::ZeroExtend },
        lir::ValType::I32 => wir::LoadKind::I32 { atomic: false },
        lir::ValType::I64 => wir::LoadKind::I64 { atomic: false },
        lir::ValType::F64 => wir::LoadKind::F64,
        lir::ValType::Ptr => wir::LoadKind::I32 { atomic: false },
        lir::ValType::FnPtr => wir::LoadKind::I32 { atomic: false },
    }
}

fn infer_store_kind(r#type: lir::ValType) -> wir::StoreKind {
    match r#type {
        lir::ValType::I8 => wir::StoreKind::I32_8 { atomic: false },
        lir::ValType::I32 => wir::StoreKind::I32 { atomic: false },
        lir::ValType::I64 => wir::StoreKind::I64 { atomic: false },
        lir::ValType::F64 => wir::StoreKind::F64,
        lir::ValType::Ptr => wir::StoreKind::I32 { atomic: false },
        lir::ValType::FnPtr => wir::StoreKind::I32 { atomic: false },
    }
}

fn infer_mem_arg(r#type: lir::ValType, offset: usize) -> wir::MemArg {
    let offset: u32 = offset.try_into().unwrap();
    match r#type {
        lir::ValType::I8 => wir::MemArg { align: 1, offset },
        lir::ValType::I32 => wir::MemArg { align: 4, offset },
        lir::ValType::I64 => wir::MemArg { align: 8, offset },
        lir::ValType::F64 => wir::MemArg { align: 8, offset },
        lir::ValType::Ptr => wir::MemArg { align: 4, offset },
        lir::ValType::FnPtr => wir::MemArg { align: 4, offset },
    }
}

fn translate_unary_op(op: lir::UnaryOpcode) -> wir::UnaryOp {
    match op {
        lir::UnaryOpcode::I64ToI32 => wir::UnaryOp::I32WrapI64,
        lir::UnaryOpcode::FNeg => wir::UnaryOp::F64Neg,
        lir::UnaryOpcode::Not => wir::UnaryOp::I32Eqz,
    }
}

fn translate_binary_op(
    op: lir::BinaryOpcode,
    lhs_type: lir::ValType,
    rhs_type: lir::ValType,
) -> wir::BinaryOp {
    use lir::BinaryOpcode as O;
    use lir::ValType as V;
    use wir::BinaryOp as Wo;
    match (op, lhs_type, rhs_type) {
        (O::IAdd, V::I32, V::I32) => Wo::I32Add,
        (O::ISub, V::I32, V::I32) => Wo::I32Sub,
        (O::IMul, V::I32, V::I32) => Wo::I32Mul,
        (O::SLt, V::I32, V::I32) => Wo::I32LtS,
        (O::SGt, V::I32, V::I32) => Wo::I32GtS,
        (O::ULt, V::I32, V::I32) => Wo::I32LtU,
        (O::UGt, V::I32, V::I32) => Wo::I32GtU,
        (O::IEq, V::I32, V::I32) => Wo::I32Eq,
        (O::INeq, V::I32, V::I32) => Wo::I32Ne,
        (O::FAdd, V::F64, V::F64) => Wo::F64Add,
        (O::FSub, V::F64, V::F64) => Wo::F64Sub,
        (O::FMul, V::F64, V::F64) => Wo::F64Mul,
        (O::FLt, V::F64, V::F64) => Wo::F64Lt,
        (O::FGt, V::F64, V::F64) => Wo::F64Gt,
        (O::FEq, V::F64, V::F64) => Wo::F64Eq,
        (O::FDiv, V::F64, V::F64) => Wo::F64Div,
        (O::FGte, V::F64, V::F64) => Wo::F64Ge,
        (O::And, V::I8, V::I8) => Wo::I32And,
        (O::Or, V::I8, V::I8) => Wo::I32Or,
        _ => unimplemented!(
            "unknown combination of op and val types: {:?}, {:?}, {:?}",
            op,
            lhs_type,
            rhs_type
        ),
    }
}

fn translate_instr_seq_type(
    module_types: &mut walrus::ModuleTypes,
    inputs: impl Iterator<Item = lir::ValType>,
    outputs: impl Iterator<Item = lir::ValType>,
) -> wir::InstrSeqType {
    let inputs: Vec<_> = inputs.map(translate_val_type).collect();
    let outputs: Vec<_> = outputs.map(translate_val_type).collect();
    if inputs.is_empty() && outputs.len() <= 1 {
        wir::InstrSeqType::Simple(outputs.first().copied())
    } else {
        let type_id = module_types.add(&inputs, &outputs);
        wir::InstrSeqType::MultiValue(type_id)
    }
}
