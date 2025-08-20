//! TODO add documentation for how the Wasm types interact with LIR types.
//! I'm writing this with the understanding that if an instruction has a type
//! like I8 which does not correspond to a Wasm type, then it will correspond
//! to its promotion, the Wasm type I32.

use std::{cell::RefCell, collections::HashMap};

use typed_index_collections::TiVec;
use walrus::ir as wir;

use crate::{lir, stackify::StackManipulators};

mod stackify_lir;

pub struct CodegenModuleCtx<'a> {
    module: &'a mut walrus::Module,
    memory_id: walrus::MemoryId,
}

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

pub fn add_function(mod_ctx: &mut CodegenModuleCtx, func: &lir::Function) {
    // create the function builder
    let parameter_types: Vec<_> =
        func.parameter_types.iter().copied().map(translate_val_type).collect();
    let return_types: Vec<_> =
        func.body.output_type.as_ref().iter().copied().map(translate_val_type).collect();
    let mut function =
        walrus::FunctionBuilder::new(&mut mod_ctx.module.types, &parameter_types, &return_types);

    // calculate stackification and other metadata about the instructions
    // let mut remaining_uses = lir::count_uses(&func.body.body);
    let types = lir::infer_output_types(func);
    let stackification = stackify_lir::stackify_cfg(func);
    let mut remaining_getters = stackify_lir::count_getters(&stackification);

    // add stack pointer if needed
    let needs_stack_ptr = true; // TODO should just check if we ever load/store to the stack
    let stack_ptr: Option<walrus::LocalId> =
        needs_stack_ptr.then(|| mod_ctx.module.locals.add(translate_val_type(lir::ValType::Ptr)));

    // associates each InsnSeqId with its walrus InstrSeqId
    let mut compound_labels = HashMap::new();

    // associates each instruction with the walrus local variable that holds its
    // output value, if one exists. using stackification, we are able to only
    // create local variables for instructions whose outputs are used
    // non-immediately.
    // TODO add function arguments to the local_ids map
    let mut local_ids = HashMap::new();
    // TODO we can make more efficient use of local variables by having the
    // ctx.uses become ctx.remaining_uses, which counts down each time a getter
    // for the value is taken. if a value is known to not be used again, we can
    // put the local variable into a pool of unused local variables, which we'll
    // pull from when we need a new local variable without having to create a
    // new one every time.

    write_code(
        &mut CodegenFnCtx {
            module: mod_ctx.module,
            func,
            compound_labels: &mut compound_labels,
            local_ids: &mut local_ids,
            remaining_getters: &mut remaining_getters,
            memory: mod_ctx.memory_id,
            stack_ptr,
            types: &types,
            stk: &stackification,
        },
        &mut function.func_body(),
        func.body.body,
    );

    /// Context for codegen that does not depend on the current instruction
    /// sequence being built, but applies to the entire function.
    struct CodegenFnCtx<'a> {
        module: &'a mut walrus::Module,
        func: &'a lir::Function,
        compound_labels: &'a mut HashMap<lir::InsnSeqId, wir::InstrSeqId>,
        local_ids: &'a mut HashMap<lir::ValRef, wir::LocalId>,
        remaining_getters: &'a mut HashMap<lir::ValRef, usize>,
        memory: walrus::MemoryId,
        stack_ptr: Option<walrus::LocalId>,
        // remaining_uses: &'a mut TiVec<lir::ValRef, usize>,
        types: &'a HashMap<lir::ValRef, lir::ValType>,
        stk: &'a stackify_lir::CfgStackification,
    }
    fn write_code(
        ctx: &mut CodegenFnCtx,
        insn_builder: &mut walrus::InstrSeqBuilder,
        insn_seq_id: lir::InsnSeqId,
    ) {
        ctx.compound_labels.insert(insn_seq_id, insn_builder.id());

        // track the operand stack. this is used so that we know what values we
        // are capturing (if they need to be stored in a local or just dropped),
        // as well as for validation
        let mut op_stack: Vec<lir::ValRef> = Vec::new();

        // the size of the prefix of the operand stack where all operands
        // needing saving are known to be saved
        let mut known_saved = 0;

        let seq_stk = &ctx.stk.seqs[insn_seq_id];
        for (idx, insn) in ctx.func.insn_seqs[insn_seq_id].iter_enumerated() {
            let pc = lir::InsnPc(insn_seq_id, idx);
            let StackManipulators { captures, getters, inputs, outputs } = &seq_stk.manips[idx];

            // generate code to handle capturing and saving before this
            // instruction executes
            for _ in 0..*captures {
                let captured = op_stack.pop().expect("stackification should be correct");

                if !ctx.local_ids.contains_key(&captured)
                    && ctx.remaining_getters.contains_key(&captured)
                {
                    let local_id = allocate_local(ctx, captured);

                    // add an instruction to store the captured value into a
                    // local variable
                    insn_builder.local_set(local_id);
                } else {
                    // drop the captured value
                    insn_builder.drop();
                }
            }
            if known_saved < op_stack.len() {
                // the previous instruction introduced some new values that
                // might need to be saved. If the operand stack looks like
                // AAABABA, where A is an operand that does not need saving and
                // B is one that does, then we need to pop as values as it takes
                // to get the leftmost B on top, then tee that B, then push back
                // all the other operands. for the common case where there is
                // just a single value which needs saving, required_pops will be
                // Some(0) and we will just tee that value.
                let required_pops = op_stack
                    .iter()
                    .rev() // iterator counting backwards
                    .take(op_stack.len() - known_saved) // stop when we reach known saved values
                    .enumerate() // topmost val is 0, second top is 1, etc.
                    .filter_map(|(i, v)| {
                        (ctx.remaining_getters.contains_key(v) && !ctx.local_ids.contains_key(v))
                            .then(|| i)
                    })
                    .last(); // get the bottomost value needing saving
                // if required_pops is None, that means we did not find any
                // new values that needed to be saved
                if let Some(required_pops) = required_pops {
                    // add instructions to pop values from the stack
                    for i in 0..required_pops {
                        let local_id = allocate_local(ctx, op_stack[op_stack.len() - i - 1]);
                        insn_builder.local_set(local_id);
                    }
                    // the bottommost value needing saving is now on top of the
                    // stack
                    let local_id =
                        allocate_local(ctx, op_stack[op_stack.len() - required_pops - 1]);
                    insn_builder.local_tee(local_id);
                    // add instructions to push the saved values back onto the
                    // stack
                    for i in (op_stack.len() - required_pops)..op_stack.len() {
                        insn_builder.local_get(ctx.local_ids[&op_stack[i]]);
                    }
                }
            }
            // now we know every operand currently on the stack is saved (if it
            // needs to be).

            // generate code to add required getters
            for v in getters {
                // update bookkeeping on how many getters are left for this value
                let num_remaining = ctx.remaining_getters.get_mut(v).unwrap();
                *num_remaining -= 1;
                if *num_remaining == 0 {
                    ctx.remaining_getters.remove(v);
                }

                // generate code
                insn_builder.local_get(ctx.local_ids[v]);
            }

            // generate code to execute the instruction
            use lir::InsnKind as I;
            match insn {
                I::FunctionArgs { output_type } => {
                    // arguments do not show up in codegen. However, any future
                    // instructions that use this argument need to know that
                    // they can get it using a local getter.
                    for i in 0..output_type.as_ref().len() {
                        let i: u8 = i.try_into().unwrap();
                        allocate_local(ctx, lir::ValRef(pc, i));
                    }
                }
                I::LoopArg { initial_value: _ } => {
                    // arguments do not show up in codegen. Unlike function
                    // args, however, loop args do output values onto the
                    // operand stack, so things are automatically handled.
                }
                I::Const { r#type, value } => {
                    insn_builder.const_(translate_val(*r#type, *value));
                }
                I::DeriveField { offset, ptr: _ } => {
                    insn_builder.const_(translate_usize(*offset)).binop(wir::BinaryOp::I32Add);
                }
                I::DeriveElement { element_size, ptr: _, index: _ } => {
                    insn_builder
                        .const_(translate_usize(*element_size))
                        .binop(wir::BinaryOp::I32Mul)
                        .binop(wir::BinaryOp::I32Add);
                }
                I::MemLoad { r#type, offset, ptr: _ } => {
                    let load_kind = infer_load_kind(*r#type);
                    let mem_arg = infer_mem_arg(*r#type, *offset);
                    insn_builder.load(ctx.memory, load_kind, mem_arg);
                }
                I::MemStore { offset, ptr: _, value } => {
                    let r#type = ctx.types[value];
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
                    let r#type = ctx.types[value];
                    let store_kind = infer_store_kind(r#type);
                    let mem_arg = infer_mem_arg(r#type, *offset);
                    insn_builder
                        .local_get(ctx.stack_ptr.expect("the presence of a StackStore instruction means there must be a stack pointer local var"))
                        .store(ctx.memory, store_kind, mem_arg);
                }
                I::CallImportedFunction { .. } => todo!(),
                I::CallUserFunction { .. } => todo!(),
                I::UnaryOp { op, operand: _ } => {
                    insn_builder.unop(translate_unary_op(*op));
                }
                I::BinaryOp { op, lhs, rhs } => {
                    insn_builder.binop(translate_binary_op(*op, ctx.types[lhs], ctx.types[rhs]));
                }
                I::Break { target, values: _ } => {
                    let target = ctx.compound_labels[target];
                    insn_builder.br(target);
                }
                I::ConditionalBreak { target, condition: _, values: _ } => {
                    let target = ctx.compound_labels[target];
                    insn_builder.br_if(target);
                }
                I::Block(lir::Block { output_type, body }) => {
                    let seq_type = translate_instr_seq_type(
                        &mut ctx.module.types,
                        ctx.stk.seqs[*body].inputs.iter().map(|v| ctx.types[v]),
                        output_type.as_ref().iter().copied(),
                    );

                    insn_builder.block(seq_type, |inner_builder| {
                        write_code(ctx, inner_builder, *body);
                    });
                }
                I::IfElse(lir::IfElse { condition: _, then_body, else_body, output_type }) => {
                    let seq_type = translate_instr_seq_type(
                        &mut ctx.module.types,
                        ctx.stk.seqs[*then_body].inputs.iter().map(|v| ctx.types[v]),
                        output_type.as_ref().iter().copied(),
                    );

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
                            write_code(&mut *ctx, then_builder, *then_body);
                        },
                        |else_builder| {
                            // this may override the label in the other branch. this
                            // is okay as long as the label is set correctly while
                            // this branch is being built.
                            let mut ctx = ctx.borrow_mut();
                            write_code(&mut *ctx, else_builder, *else_body);
                        },
                    );
                }
                I::Loop(lir::Loop { body, inputs, output_type }) => {
                    let seq_type = translate_instr_seq_type(
                        &mut ctx.module.types,
                        inputs.iter().map(|v| ctx.types[v]),
                        output_type.as_ref().iter().copied(),
                    );

                    insn_builder.loop_(seq_type, |inner_builder| {
                        write_code(ctx, inner_builder, *body);
                    });
                }
            };

            // remove the inputs from our symbolic operand stack
            for input in inputs.iter().rev() {
                let consumed = op_stack.pop();
                assert_eq!(consumed, Some(*input));
            }
            known_saved = op_stack.len();
            // add the outputs to our symbolic operand stack
            let outputs = outputs;
            op_stack.extend(outputs);
        }

        // TODO should we deliberately skip stack manipulators at the end? the
        // end idx won't show up because it's not a real instruction
    }
    fn allocate_local(ctx: &mut CodegenFnCtx, val: lir::ValRef) -> wir::LocalId {
        let local_id = ctx.module.locals.add(translate_val_type(ctx.types[&val]));
        ctx.local_ids.insert(val, local_id);
        local_id
    }

    // TODO add function arguments
    function.finish(vec![], &mut mod_ctx.module.funcs);
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
    use crate::lir::lir_function;

    #[test]
    fn empty_module() {
        let lir = lir::Program::default();
        let mut module = lir_to_wasm(&lir);
        let wasm = module.emit_wasm();
        std::fs::write("test.wasm", wasm).unwrap();
    }

    #[test]
    fn return_1() {
        lir_function! {
            fn func() -> (I32) main: {
                break_(main)(constant(I32, 10));
            }
        }
        let mut lir = lir::Program::default();
        let func_id = lir.functions.push_and_get_key(func);
        lir.entrypoints.push(func_id);

        let mut module = lir_to_wasm(&lir);
        let wasm = module.emit_wasm();
        std::fs::write("test.wasm", wasm).unwrap();
    }
}
