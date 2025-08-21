//! TODO add documentation for how the Wasm types interact with LIR types.
//! I'm writing this with the understanding that if an instruction has a type
//! like I8 which does not correspond to a Wasm type, then it will correspond
//! to its promotion, the Wasm type I32.

use std::{cell::RefCell, collections::HashMap};

use walrus::ir as wir;

use crate::{
    stackify_generic::StackManipulators,
    stackify_lir::{self, ValRefOrStackPtr},
};

struct CodegenModuleCtx<'a, A> {
    /// The module being generated.
    module: walrus::Module,
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
    /// A map from LIR imported function ids to Walrus function ids.
    imported_fn_ids: HashMap<lir::ImportedFunctionId, walrus::FunctionId>,
}

pub trait FnTableSlotAllocator {
    /// Return a free slot in the function table.
    fn allocate_slot(&mut self) -> usize;
}

/// Translates a LIR program into a Walrus module. Returns the generated module
/// as well as a map from the LIR function id to the slot in the function table.
pub fn lir_to_wasm(
    lir: &lir::Program,
    fn_table_slot_allocator: &mut impl FnTableSlotAllocator,
) -> (walrus::Module, HashMap<lir::FunctionId, usize>) {
    // create the module
    let config = walrus::ModuleConfig::default();
    let mut module = walrus::Module::with_config(config);

    // import memory, stack pointer, and function table
    #[rustfmt::skip]
    let (mem_id, _mem_import_id) = module.add_import_memory(
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
    let (sp_global_id, _stack_ptr_import_id) = module.add_import_global(
        "env",
        "__stack_pointer",
        translate_val_type(lir::ValType::Ptr),
        true,
        // not shared
        false,
    );
    let (fn_table_id, _fn_table_import_id) = module.add_import_table(
        "env",
        "__indirect_function_table",
        // table64
        false,
        // initial table size. I think 0 means any size above 0
        0,
        // max table
        None,
        walrus::RefType::Funcref,
    );

    // add imported functions
    let mut imported_fn_ids = HashMap::new();
    for (imported_function_id, imported_function) in lir.imported_functions.iter_enumerated() {
        let param_types: Vec<_> =
            imported_function.parameter_types.iter().copied().map(translate_val_type).collect();
        let return_types: Vec<_> =
            imported_function.return_type.iter().copied().map(translate_val_type).collect();
        let func_type = module.types.add(&param_types, &return_types);
        let (w_func_id, _) = module.add_import_func("env", imported_function.name, func_type);
        imported_fn_ids.insert(imported_function_id, w_func_id);
    }

    let mut ctx = CodegenModuleCtx {
        module,
        mem_id,
        sp_global_id,
        fn_table_id,
        fn_table_slot_allocator,
        fn_table_allocated_slots: HashMap::new(),
        user_fn_ids: HashMap::new(),
        imported_fn_ids,
    };

    // allocate slots in the function table for entrypoint functions
    for entrypoint in &lir.entrypoints {
        let slot = ctx.fn_table_slot_allocator.allocate_slot();
        ctx.fn_table_allocated_slots.insert(*entrypoint, slot);
    }

    // add user functions. these may allocate additional slots in the function
    // table for callbacks, every time the address of a function is taken.
    for (lir_fid, function) in lir.user_functions.iter_enumerated() {
        let wir_fid = add_function(&mut ctx, function);
        ctx.user_fn_ids.insert(lir_fid, wir_fid);
    }

    // use element segments to install all functions for which fn table slots
    // were allocated
    for (lir_fn_id, slot) in &ctx.fn_table_allocated_slots {
        let wir_fn_id = ctx.user_fn_ids[lir_fn_id];
        ctx.module.elements.add(
            walrus::ElementKind::Active {
                table: ctx.fn_table_id,
                offset: walrus::ConstExpr::Value(translate_usize(*slot)),
            },
            walrus::ElementItems::Functions(vec![wir_fn_id]),
        );
    }

    (ctx.module, ctx.fn_table_allocated_slots)
}

fn add_function<A: FnTableSlotAllocator>(
    mod_ctx: &mut CodegenModuleCtx<A>,
    func: &lir::Function,
) -> walrus::FunctionId {
    /// Context for codegen that applies to the entire function.
    struct CodegenFnCtx<'a, 'b, A> {
        /// The context of the module being generated.
        mod_ctx: &'b mut CodegenModuleCtx<'a, A>,
        /// The function whose code is being generated.
        func: &'b lir::Function,
        /// The types of each LIR value
        types: &'b HashMap<lir::ValRef, lir::ValType>,
        /// Metadata for stackifying the function.
        stk: &'b stackify_lir::CfgStackification,
        /// Associates each LIR insn seq id with the walrus insn seq id.
        compound_labels: HashMap<lir::InsnSeqId, wir::InstrSeqId>,
        /// Associates each LIR value with the walrus local variable, if one
        /// exists.
        local_ids: HashMap<lir::ValRef, wir::LocalId>,
        /// The walrus local variable that holds the stack pointer.
        sp_local_id: Option<wir::LocalId>,
        /// The walrus local variables that hold the function arguments.
        arg_locals: Vec<wir::LocalId>,
        /// The number of getters remaining for each LIR value.
        remaining_getters: HashMap<lir::ValRef, usize>,
    }

    // create the function builder
    let parameter_types: Vec<_> =
        func.parameter_types.iter().copied().map(translate_val_type).collect();
    let return_types: Vec<_> =
        func.body.output_type.iter().copied().map(translate_val_type).collect();
    let mut function =
        walrus::FunctionBuilder::new(&mut mod_ctx.module.types, &parameter_types, &return_types);

    // add stack pointer if needed
    let needs_stack_ptr = func.stack_space > 0;
    let sp_local_id: Option<walrus::LocalId> =
        needs_stack_ptr.then(|| mod_ctx.module.locals.add(translate_val_type(lir::ValType::Ptr)));

    // TODO we can make more efficient use of local variables by having the
    // ctx.uses become ctx.remaining_uses, which counts down each time a getter
    // for the value is taken. if a value is known to not be used again, we can
    // put the local variable into a pool of unused local variables, which we'll
    // pull from when we need a new local variable without having to create a
    // new one every time.

    let types = lir::infer_output_types(func);
    let stk = stackify_lir::stackify_cfg(func);

    // create the context for generating the function's code
    let mut ctx = CodegenFnCtx {
        mod_ctx,
        func,
        types: &types,
        stk: &stk,
        compound_labels: HashMap::new(),
        local_ids: HashMap::new(),
        sp_local_id,
        arg_locals: Vec::new(),
        remaining_getters: stackify_lir::count_getters(&stk),
    };

    if let Some(stack_ptr_local) = sp_local_id {
        // there is a stack pointer. generate a prologue and epilogue that
        // initializes the stack pointer

        let mut insn_builder = function.func_body();
        // subtract from the stack pointer
        insn_builder
            .global_get(ctx.mod_ctx.sp_global_id)
            .const_(translate_usize(func.stack_space))
            .binop(wir::BinaryOp::I32Sub)
            .local_tee(stack_ptr_local)
            .global_set(ctx.mod_ctx.sp_global_id);

        // put the function body in a block
        insn_builder.block(
            translate_instr_seq_type(
                &mut ctx.mod_ctx.module.types,
                std::iter::empty(),
                func.body.output_type.iter().copied(),
            ),
            |inner_builder| {
                write_code(&mut ctx, inner_builder, func.body.body);
            },
        );

        // add to the stack pointer
        insn_builder
            .local_get(stack_ptr_local)
            .const_(translate_usize(func.stack_space))
            .binop(wir::BinaryOp::I32Add)
            .global_set(ctx.mod_ctx.sp_global_id);
    } else {
        // there is no stack pointer, so no need for prologue or epilogue
        write_code(&mut ctx, &mut function.func_body(), func.body.body);
    }
    fn write_code<A: FnTableSlotAllocator>(
        ctx: &mut CodegenFnCtx<A>,
        insn_builder: &mut walrus::InstrSeqBuilder,
        insn_seq_id: lir::InsnSeqId,
    ) {
        ctx.compound_labels.insert(insn_seq_id, insn_builder.id());

        // track the operand stack. this is used so that we know what values we
        // are capturing (if they need to be stored in a local or just dropped),
        // as well as for validation
        let mut op_stack: Vec<ValRefOrStackPtr> = Vec::new();

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
                let ValRefOrStackPtr::ValRef(captured) = captured else {
                    panic!("stackification should not cause stack ptr operand to be captured");
                };

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
                        // stackification should not cause stack ptr to be part of a multivalue
                        let v = v.unwrap_val_ref();
                        (ctx.remaining_getters.contains_key(&v) && !ctx.local_ids.contains_key(&v))
                            .then(|| i)
                    })
                    .last(); // get the bottomost value needing saving
                // if required_pops is None, that means we did not find any
                // new values that needed to be saved
                if let Some(required_pops) = required_pops {
                    // add instructions to pop values from the stack
                    for i in 0..required_pops {
                        let local_id =
                            allocate_local(ctx, op_stack[op_stack.len() - i - 1].unwrap_val_ref());
                        insn_builder.local_set(local_id);
                    }
                    // the bottommost value needing saving is now on top of the
                    // stack
                    let local_id = allocate_local(
                        ctx,
                        op_stack[op_stack.len() - required_pops - 1].unwrap_val_ref(),
                    );
                    insn_builder.local_tee(local_id);
                    // add instructions to push the saved values back onto the
                    // stack
                    for i in (op_stack.len() - required_pops)..op_stack.len() {
                        insn_builder.local_get(ctx.local_ids[&op_stack[i].unwrap_val_ref()]);
                    }
                }
            }
            // now we know every operand currently on the stack is saved (if it
            // needs to be).

            // generate code to add required getters
            for v in getters {
                match v {
                    ValRefOrStackPtr::ValRef(v) => {
                        // update bookkeeping on how many getters are left for this value
                        let num_remaining = ctx.remaining_getters.get_mut(v).unwrap();
                        *num_remaining -= 1;
                        if *num_remaining == 0 {
                            ctx.remaining_getters.remove(v);
                        }

                        // generate code
                        insn_builder.local_get(ctx.local_ids[v]);
                    }
                    ValRefOrStackPtr::StackPtr => {
                        insn_builder.local_get(ctx.sp_local_id.expect("the presence of a StackLoad instruction means there must be a stack pointer local var"));
                    }
                }
                // update op stack to reflect that the value is now available
                op_stack.push(*v);
            }

            // generate code to execute the instruction
            use lir::InsnKind as I;
            match insn {
                I::FunctionArgs { output_type } => {
                    // arguments do not show up in codegen. However, any future
                    // instructions that use this argument need to know that
                    // they can get it using a local getter.
                    assert!(ctx.arg_locals.is_empty());
                    for i in 0..output_type.len() {
                        let i: u8 = i.try_into().unwrap();
                        let local_id = allocate_local(ctx, lir::ValRef(pc, i));
                        ctx.arg_locals.push(local_id);
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
                I::UserFunctionPtr { function } => {
                    let slot = addr_of_function(ctx, *function);
                    insn_builder.const_(translate_usize(slot));
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
                    insn_builder.load(ctx.mod_ctx.mem_id, load_kind, mem_arg);
                }
                I::MemStore { offset, ptr: _, value } => {
                    let r#type = ctx.types[value];
                    let store_kind = infer_store_kind(r#type);
                    let mem_arg = infer_mem_arg(r#type, *offset);
                    insn_builder.store(ctx.mod_ctx.mem_id, store_kind, mem_arg);
                }
                I::StackLoad { r#type, offset } => {
                    let load_kind = infer_load_kind(*r#type);
                    let mem_arg = infer_mem_arg(*r#type, *offset);
                    insn_builder
                        .local_get(ctx.sp_local_id.expect("the presence of a StackAddr instruction means there must be a stack pointer local var"))
                        .load(ctx.mod_ctx.mem_id, load_kind, mem_arg);
                }
                I::StackStore { offset, value } => {
                    let r#type = ctx.types[value];
                    let store_kind = infer_store_kind(r#type);
                    let mem_arg = infer_mem_arg(r#type, *offset);
                    // stackification ensured that the stack pointer, followed
                    // by the value, are already on the operand stack. we just
                    // need to emit the store instruction.
                    insn_builder.store(ctx.mod_ctx.mem_id, store_kind, mem_arg);
                }
                I::StackAddr { offset } => {
                    insn_builder
                        .local_get(ctx.sp_local_id.expect("the presence of a StackAddr instruction means there must be a stack pointer local var"))
                        .const_(translate_usize(*offset))
                        .binop(wir::BinaryOp::I32Add);
                }
                I::CallImportedFunction { function, args: _, output_type: _ } => {
                    let callee = ctx.mod_ctx.imported_fn_ids[function];
                    insn_builder.call(callee);
                }
                I::CallUserFunction { function, args: _, output_type: _ } => {
                    let callee = ctx.mod_ctx.user_fn_ids[function];
                    insn_builder.call(callee);
                }
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
                        &mut ctx.mod_ctx.module.types,
                        ctx.stk.seqs[*body].inputs.iter().map(|v| ctx.types[&v.unwrap_val_ref()]),
                        output_type.iter().copied(),
                    );

                    insn_builder.block(seq_type, |inner_builder| {
                        write_code(ctx, inner_builder, *body);
                    });
                }
                I::IfElse(lir::IfElse { condition: _, then_body, else_body, output_type }) => {
                    let seq_type = translate_instr_seq_type(
                        &mut ctx.mod_ctx.module.types,
                        ctx.stk.seqs[*then_body]
                            .inputs
                            .iter()
                            .map(|v| ctx.types[&v.unwrap_val_ref()]),
                        output_type.iter().copied(),
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
                        &mut ctx.mod_ctx.module.types,
                        inputs.iter().map(|v| ctx.types[v]),
                        output_type.iter().copied(),
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
    fn allocate_local<A>(ctx: &mut CodegenFnCtx<A>, val: lir::ValRef) -> wir::LocalId {
        let local_id = ctx.mod_ctx.module.locals.add(translate_val_type(ctx.types[&val]));
        ctx.local_ids.insert(val, local_id);
        local_id
    }
    fn addr_of_function<A: FnTableSlotAllocator>(
        ctx: &mut CodegenFnCtx<A>,
        function: lir::FunctionId,
    ) -> usize {
        use std::collections::hash_map::Entry;
        match ctx.mod_ctx.fn_table_allocated_slots.entry(function) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let new_slot = ctx.mod_ctx.fn_table_slot_allocator.allocate_slot();
                entry.insert(new_slot);
                new_slot
            }
        }
    }

    function.finish(ctx.arg_locals, &mut mod_ctx.module.funcs)
}

/// Translate a LIR value type to a Wasm value type.
fn translate_val_type(r#type: lir::ValType) -> walrus::ValType {
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
fn translate_val(r#type: lir::ValType, value: u64) -> walrus::ir::Value {
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
fn translate_usize(x: usize) -> walrus::ir::Value {
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
    use lir::lir_function;

    use super::*;

    #[derive(Default)]
    struct TestFnTableSlotAllocator {
        next_slot: usize,
    }
    impl FnTableSlotAllocator for TestFnTableSlotAllocator {
        fn allocate_slot(&mut self) -> usize {
            let slot = self.next_slot;
            self.next_slot += 1;
            slot
        }
    }

    #[test]
    fn empty_module() {
        let lir = lir::Program::default();
        let (mut module, _) = lir_to_wasm(&lir, &mut TestFnTableSlotAllocator::default());
        let wasm = module.emit_wasm();
        std::fs::write("test.wasm", wasm).unwrap();
    }

    #[test]
    fn return_1() {
        lir_function! {
            fn func() -> (I32),
            stack_space: 0,
            main: {
                break_(main)(constant(I32, 10));
            }
        }
        let mut lir = lir::Program::default();
        let func_id = lir.user_functions.push_and_get_key(func);
        lir.entrypoints.push(func_id);

        let (mut module, _) = lir_to_wasm(&lir, &mut TestFnTableSlotAllocator::default());
        let wasm = module.emit_wasm();
        std::fs::write("test.wasm", wasm).unwrap();
    }

    #[test]
    fn add_one() {
        lir_function! {
            fn add_one(I32) -> (I32),
            stack_space: 0,
            main: {
                %arg = arguments(-> (I32));
                %res = Add(arg, constant(I32, 1));
                break_(main)(res);
            }
        }
        let mut lir = lir::Program::default();
        let func_id = lir.user_functions.push_and_get_key(add_one);
        lir.entrypoints.push(func_id);

        let (mut module, _) = lir_to_wasm(&lir, &mut TestFnTableSlotAllocator::default());
        let wasm = module.emit_wasm();
        std::fs::write("test.wasm", wasm).unwrap();
    }

    #[test]
    fn use_stack() {
        lir_function! {
            fn my_function(I32) -> (I32),
            stack_space: 16,
            main: {
                %arg = arguments(-> (I32));
                stack_store(8)(arg);
                %_res = stack_load(I16, 8);
                break_(main)(stack_addr(8));
            }
        }

        let mut lir = lir::Program::default();
        let func_id = lir.user_functions.push_and_get_key(my_function);
        lir.entrypoints.push(func_id);

        let (mut module, _) = lir_to_wasm(&lir, &mut TestFnTableSlotAllocator::default());
        let wasm = module.emit_wasm();
        std::fs::write("test.wasm", wasm).unwrap();
    }

    #[test]
    fn call_imported_function() {
        let mut lir = lir::Program::default();
        let imported_func_id = lir.imported_functions.push_and_get_key(lir::ImportedFunction {
            name: "defined_elsewhere",
            parameter_types: vec![lir::ValType::I32, lir::ValType::F64],
            return_type: vec![lir::ValType::F64, lir::ValType::I32],
        });

        lir_function! {
            fn my_function() -> (F64, I32),
            stack_space: 0,
            main: {
                %(a, b) = call_imported_function(imported_func_id -> (F64, I32))(
                    constant(I32, 10),
                    constant(F64, 20)
                );
                break_(main)(a, b);
            }
        }
        let func_id = lir.user_functions.push_and_get_key(my_function);
        lir.entrypoints.push(func_id);

        let (mut module, _) = lir_to_wasm(&lir, &mut TestFnTableSlotAllocator { next_slot: 0 });
        let wasm = module.emit_wasm();
        std::fs::write("test.wasm", wasm).unwrap();
    }

    #[test]
    fn call_user_function() {
        let mut lir = lir::Program::default();
        lir_function! {
            fn callee(F64) -> (I32),
            stack_space: 0,
            main: {
                break_(main)(constant(I32, 10));
            }
        }
        let callee_id = lir.user_functions.push_and_get_key(callee);
        lir_function! {
            fn caller() -> (I32),
            stack_space: 0,
            main: {
                %a = call_user_function(callee_id -> (I32))(constant(F64, 1));
                break_(main)(a);
            }
        }
        let caller_id = lir.user_functions.push_and_get_key(caller);
        lir.entrypoints.push(caller_id);

        let (mut module, _) = lir_to_wasm(&lir, &mut TestFnTableSlotAllocator::default());
        let wasm = module.emit_wasm();
        std::fs::write("test.wasm", wasm).unwrap();
    }
}
