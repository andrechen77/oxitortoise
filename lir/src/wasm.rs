//! TODO add documentation for how the Wasm types interact with LIR types.
//! I'm writing this with the understanding that if an instruction has a type
//! like I8 which does not correspond to a Wasm type, then it will correspond
//! to its promotion, the Wasm type I32.

use std::{cell::RefCell, collections::HashMap, ops::Range};

use typed_index_collections::{TiVec, ti_vec};
use walrus::ir as wir;

use crate::{
    lir,
    stackify::OutputMode,
    wasm::stackify_lir::{Stackification, stackify_lir},
};

mod stackify_lir;

pub fn lir_to_wasm(lir: &lir::Program) -> walrus::Module {
    let config = walrus::ModuleConfig::default();

    let mut module = walrus::Module::with_config(config);
    #[rustfmt::skip]
    let (memory_id, _mem_import_id) = module.add_import_memory(
        "env",
        "memory",
        // not shared. even though we are importing a memory, it's not shared because we are
        // not using threads
        false,
        // not 64-bit memory
        false,
        // initial memory size. I think 0 means any amount of memory above 0
        0,
        None,
        None,
    );
    // TODO also import stack pointer and indirect function table.

    let mut ctx = CodegenModuleCtx { module: &mut module, memory_id };

    for function in &lir.functions {
        add_function(&mut ctx, function);
    }

    // TODO add export for entrypoint functions

    module
}

pub struct CodegenModuleCtx<'a> {
    module: &'a mut walrus::Module,
    memory_id: walrus::MemoryId,
}

#[allow(unreachable_code)] // for the fact that i don't wanna get the memory
pub fn add_function(ctx: &mut CodegenModuleCtx, lir: &lir::Function) {
    // create the function builder
    let parameter_types: Vec<_> =
        lir.parameter_types.iter().copied().map(translate_val_type).collect();
    let return_types: Vec<_> = lir.return_types.iter().copied().map(translate_val_type).collect();
    let mut function =
        walrus::FunctionBuilder::new(&mut ctx.module.types, &parameter_types, &return_types);

    // calculate stackification and other metadata about the instructions
    let lir_insns = &lir.instructions;
    let uses = lir::count_uses(lir_insns);
    let types = lir::infer_output_types(lir_insns);
    let (stackification, additional_inputs) = stackify_lir(lir_insns);

    // add stack pointer if needed
    let needs_stack_ptr = true; // TODO should just check if we ever load/store to the stack
    let stack_ptr: Option<walrus::LocalId> =
        needs_stack_ptr.then(|| ctx.module.locals.add(translate_val_type(lir::ValType::Ptr)));

    let mut insn_builder = function.func_body();

    // associates each compound instruction with its walrus InstrSeqId
    let mut compound_labels = HashMap::new();

    // associates each instruction with the walrus local variable that holds its
    // output value, if one exists. using stackification, we are able to only
    // create local variables for instructions whose outputs are used
    // non-immediately.
    let mut local_ids = HashMap::new();
    // TODO we can make more efficient use of local variables by having the
    // ctx.uses become ctx.remaining_uses, which counts down each time a getter
    // for the value is taken. if a value is known to not be used again, we can
    // put the local variable into a pool of unused local variables, which we'll
    // pull from when we need a new local variable without having to create a
    // new one every time.

    let mut ctx = CodegenFnCtx {
        module: ctx.module,
        compound_labels: &mut compound_labels,
        local_ids: &mut local_ids,
        memory: ctx.memory_id,
        stack_ptr,
        lir_insns,
        uses: &uses,
        types: &types,
        stackification: &stackification,
        additional_inputs: &additional_inputs,
    };
    write_code(&mut ctx, &mut insn_builder, lir::InsnPc(0)..lir::InsnPc(lir_insns.len()));
    // TODO there should be a special case for the first level of instructions,
    // which we expect to be just a single block that evaluates to all the
    // function's return values.

    /// Context for codegen that does not depend on the current instruction
    /// sequence being built, but applies to the entire function.
    struct CodegenFnCtx<'a> {
        module: &'a mut walrus::Module,
        compound_labels: &'a mut HashMap<lir::InsnPc, wir::InstrSeqId>,
        local_ids: &'a mut HashMap<lir::InsnPc, wir::LocalId>,
        memory: walrus::MemoryId,
        stack_ptr: Option<walrus::LocalId>,
        lir_insns: &'a TiVec<lir::InsnPc, lir::InsnKind>,
        uses: &'a TiVec<lir::InsnPc, usize>,
        types: &'a TiVec<lir::InsnPc, lir::InsnOutput>,
        stackification: &'a Stackification,
        additional_inputs: &'a HashMap<lir::InsnPc, Vec<lir::InsnPc>>,
    }
    fn write_code(
        ctx: &mut CodegenFnCtx,
        insn_builder: &mut walrus::InstrSeqBuilder,
        seq: Range<lir::InsnPc>,
    ) {
        // FUTURE add an operand stack just for validation so we know that
        // stackification went well

        for (inner_seqs, pc) in lir::InsnRefIter::new_with_range(ctx.lir_insns, seq) {
            // insert all getters for the inputs to this instruction
            if let Some(values_being_gotten) = ctx.stackification.getters.get(&pc) {
                for value_being_gotten in values_being_gotten {
                    let local_id = ctx.local_ids[value_being_gotten];
                    insn_builder.local_get(local_id);
                }
            }

            // generate code to execute the instruction
            use lir::InsnKind as I;
            match &ctx.lir_insns[pc] {
                I::Argument { .. } => {} // arguments do not show up in codegen
                I::LoopArgument { initial_value } => {} // arguments do not show up in codegen
                I::Project { .. } => {}  // projections do not show up in codegen
                I::Const { r#type, value } => {
                    insn_builder.const_(translate_val(*r#type, *value));
                }
                I::DeriveField { offset, ptr } => {
                    insn_builder.const_(translate_usize(*offset)).binop(wir::BinaryOp::I32Add);
                }
                I::DeriveElement { element_size, ptr, index } => {
                    insn_builder
                        .const_(translate_usize(*element_size))
                        .binop(wir::BinaryOp::I32Mul)
                        .binop(wir::BinaryOp::I32Add);
                }
                I::MemLoad { r#type, offset, ptr } => {
                    let load_kind = infer_load_kind(*r#type);
                    let mem_arg = infer_mem_arg(*r#type, *offset);
                    insn_builder.load(ctx.memory, load_kind, mem_arg);
                }
                I::MemStore { offset, ptr, value } => {
                    let r#type = ctx.types[*value].unwrap_single();
                    let store_kind = infer_store_kind(r#type);
                    let mem_arg = infer_mem_arg(r#type, *offset);
                    insn_builder.store(ctx.memory, store_kind, mem_arg);
                }
                I::StackLoad { r#type, offset } => {
                    let load_kind = infer_load_kind(*r#type);
                    let mem_arg = infer_mem_arg(*r#type, *offset);
                    insn_builder
                        .local_get(ctx.stack_ptr.expect("the presence of a StackLoad instruction means there must be a stack pointer local var"))
                        .load(ctx.memory, load_kind, mem_arg);
                }
                I::StackStore { offset, value } => {
                    let r#type = ctx.types[*value].unwrap_single();
                    let store_kind = infer_store_kind(r#type);
                    let mem_arg = infer_mem_arg(r#type, *offset);
                    insn_builder
                        .local_get(ctx.stack_ptr.expect("the presence of a StackStore instruction means there must be a stack pointer local var"))
                        .store(ctx.memory, store_kind, mem_arg);
                }
                I::CallImportedFunction { .. } => todo!(),
                I::CallUserFunction { .. } => todo!(),
                I::UnaryOp { op, operand } => {
                    insn_builder.unop(translate_unary_op(*op));
                }
                I::BinaryOp { op, lhs, rhs } => {
                    insn_builder.binop(translate_binary_op(
                        *op,
                        ctx.types[*lhs].unwrap_single(),
                        ctx.types[*rhs].unwrap_single(),
                    ));
                }
                I::Break { target, values } => {
                    let target = ctx.compound_labels[target];
                    insn_builder.br(target);
                }
                I::ConditionalBreak { target, condition, values } => {
                    let target = ctx.compound_labels[target];
                    insn_builder.br_if(target);
                }
                I::Block { body_len: _, output_type: _ } => {
                    let seq_type = infer_instr_seq_type(ctx, pc);
                    let [inner_seq] = &inner_seqs[..] else {
                        panic!("a block instruction must have exactly one inner sequence");
                    };
                    insn_builder.block(seq_type, |inner_builder| {
                        ctx.compound_labels.insert(pc, inner_builder.id());
                        write_code(ctx, inner_builder, inner_seq.clone());
                    });
                }
                I::IfElse { condition, then_len: _, else_len: _, output_type: _ } => {
                    let seq_type = infer_instr_seq_type(ctx, pc);
                    let [then_seq, else_seq] = &inner_seqs[..] else {
                        panic!("an if-else instruction must have exactly two inner sequences");
                    };
                    // put the &mut in a RefCell so we can borrow it twice in
                    // the two closures.
                    let ctx = RefCell::new(&mut *ctx);
                    insn_builder.if_else(
                        seq_type,
                        |then_builder| {
                            // this may override the label in the other branch. this
                            // is okay as long as the label is set correctly while
                            // this branch is being built.
                            let mut ctx = ctx.borrow_mut();
                            ctx.compound_labels.insert(pc, then_builder.id());
                            write_code(&mut *ctx, then_builder, then_seq.clone());
                        },
                        |else_builder| {
                            // this may override the label in the other branch. this
                            // is okay as long as the label is set correctly while
                            // this branch is being built.
                            let mut ctx = ctx.borrow_mut();
                            ctx.compound_labels.insert(pc, else_builder.id());
                            write_code(&mut *ctx, else_builder, else_seq.clone());
                        },
                    );
                }
                I::Loop { body_len: _, output_type: _ } => {
                    let seq_type = infer_instr_seq_type(ctx, pc);
                    let [inner_seq] = &inner_seqs[..] else {
                        panic!("a loop instruction must have exactly one inner sequence");
                    };
                    insn_builder.loop_(seq_type, |inner_builder| {
                        ctx.compound_labels.insert(pc, inner_builder.id());
                        write_code(ctx, inner_builder, inner_seq.clone());
                    });
                }
            };

            // resolve the output mode of this instruction
            match &ctx.types[pc] {
                lir::InsnOutput::Other(types) if types.is_empty() => {
                    // the instruction has no outputs, so we don't need to do
                    // anything
                    assert!(ctx.stackification.forest[pc].output_mode == OutputMode::Available);
                }
                lir::InsnOutput::Single(r#type) => {
                    match ctx.stackification.forest[pc].output_mode {
                        OutputMode::Available | OutputMode::Capture { .. } => {
                            // do not release the output value onto the stack.
                            if ctx.uses[pc] == 0 {
                                // drop the value right then and there
                                insn_builder.drop();
                            } else {
                                // store the value into a local variable
                                let local_id = allocate_local(ctx, *r#type, pc);
                                insn_builder.local_set(local_id);
                            }
                        }
                        OutputMode::Release { .. } => {
                            if ctx.uses[pc] > 1 {
                                // tee the value into a local variable
                                let local_id = allocate_local(ctx, *r#type, pc);
                                insn_builder.local_tee(local_id);
                            }
                        }
                    }
                }
                lir::InsnOutput::Other(types) => {
                    // A multivalue instruction itself doesn't actually output
                    // any values from a LIR perspective, even if the
                    // corresponding Wasm instruction does. Instead, the
                    // multivalue instruction is followed by a sequence of
                    // projection instructions which each in turn output the
                    // corresponding value, simulating what would happen if the
                    // real multivalue instruction was executed. Thus, it is
                    // actually the output mode of the projection instructions
                    // that determines how multivalues are handled. we don't
                    // need to do anything here.
                }
            }
        }
    }
    fn infer_instr_seq_type(ctx: &mut CodegenFnCtx, pc: lir::InsnPc) -> wir::InstrSeqType {
        let inputs =
            ctx.additional_inputs[&pc].iter().map(|&input_pc| ctx.types[input_pc].unwrap_single());
        let outputs = ctx.types[pc].as_ref().iter().copied();
        translate_instr_seq_type(&mut ctx.module.types, inputs, outputs)
    }
    fn allocate_local(
        ctx: &mut CodegenFnCtx,
        r#type: lir::ValType,
        pc: lir::InsnPc,
    ) -> wir::LocalId {
        let local_id = ctx.module.locals.add(translate_val_type(r#type));
        ctx.local_ids.insert(pc, local_id);
        local_id
    }
}

/// Translate a LIR value type to a Wasm value type.
pub fn translate_val_type(r#type: lir::ValType) -> walrus::ValType {
    match r#type {
        lir::ValType::I8 => walrus::ValType::I32,
        lir::ValType::I16 => walrus::ValType::I32,
        lir::ValType::I32 => walrus::ValType::I32,
        lir::ValType::I64 => walrus::ValType::I64,
        lir::ValType::F64 => walrus::ValType::F64,
        lir::ValType::Ptr => walrus::ValType::I32,
        lir::ValType::FnPtr => walrus::ValType::I32,
    }
}

/// Translate a LIR value to a Wasm value.
pub fn translate_val(r#type: lir::ValType, value: u64) -> walrus::ir::Value {
    match r#type {
        lir::ValType::I8 => wir::Value::I32(value as i32),
        lir::ValType::I16 => wir::Value::I32(value as i32),
        lir::ValType::I32 => wir::Value::I32(value as i32),
        lir::ValType::I64 => wir::Value::I64(value as i64),
        lir::ValType::F64 => wir::Value::F64(value as f64),
        // only if pointer width is 32 bits
        lir::ValType::Ptr => wir::Value::I32(value as i32),
        lir::ValType::FnPtr => wir::Value::I32(value as i32),
    }
}

/// Translates a pointer or pointer-sized value in LIR to a Wasm value. This
/// is preferred over using `as` casts because it type-checks that the input is
/// a `usize`, and picks the correct output type for the target platform.
// TODO this function depends on the fact that the pointer width is 32 bits, so
// maybe annotate this with an attribute that fails compilation if this is not
// true?
pub fn translate_usize(x: usize) -> walrus::ir::Value {
    wir::Value::I32(x as i32)
}

fn infer_load_kind(r#type: lir::ValType) -> wir::LoadKind {
    use wir::ExtendedLoad::*;
    match r#type {
        lir::ValType::I8 => wir::LoadKind::I32_8 { kind: ZeroExtend },
        lir::ValType::I16 => wir::LoadKind::I32_16 { kind: ZeroExtend },
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
        lir::ValType::I16 => wir::StoreKind::I32_16 { atomic: false },
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
        lir::ValType::I16 => wir::MemArg { align: 2, offset },
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
    }
}

fn translate_binary_op(
    op: lir::BinaryOpcode,
    lhs_type: lir::ValType,
    rhs_type: lir::ValType,
) -> wir::BinaryOp {
    use lir::BinaryOpcode::*;
    use lir::ValType::*;
    use wir::BinaryOp as Wo;
    match (op, lhs_type, rhs_type) {
        (Add, I32, I32) => Wo::I32Add,
        (Add, _, _) => panic!("unsupported binary op"),
        _ => todo!(),
    }
}

fn translate_instr_seq_type(
    module_types: &mut walrus::ModuleTypes,
    inputs: impl Iterator<Item = lir::ValType>,
    outputs: impl Iterator<Item = lir::ValType>,
) -> wir::InstrSeqType {
    let inputs: Vec<_> = inputs.map(translate_val_type).collect();
    let outputs: Vec<_> = outputs.map(translate_val_type).collect();
    if inputs.len() == 0 && outputs.len() <= 1 {
        wir::InstrSeqType::Simple(outputs.first().copied())
    } else {
        let type_id = module_types.add(&inputs, &outputs);
        wir::InstrSeqType::MultiValue(type_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions;

    #[test]
    fn empty_module() {
        let lir = lir::Program { entrypoints: vec![], functions: vec![] };
        let mut module = lir_to_wasm(&lir);
        let wasm = module.emit_wasm();
        std::fs::write("test.wasm", wasm).unwrap();
    }

    #[test]
    fn simple_function() {
        instructions! {
            let insns;

        }
    }
}
