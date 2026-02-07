//! TODO add documentation for how the Wasm types interact with LIR types.
//! I'm writing this with the understanding that if an instruction has a type
//! like I8 which does not correspond to a Wasm type, then it will correspond
//! to its promotion, the Wasm type I32.

use std::{cell::RefCell, collections::HashMap};

use tracing::{debug, info, trace};
use walrus::ir as wir;

use crate::{
    stackify_generic::StackManipulators,
    stackify_lir::{self, ValRefOrStackPtr},
};

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

/// Translates a LIR program into a Walrus module. Returns the generated module
/// as well as a map from the LIR function id to the slot in the function table.
pub fn lir_to_wasm(
    lir: &lir::Program,
    fn_table_slot_allocator: &mut impl FnTableSlotAllocator,
) -> (walrus::Module, HashMap<lir::FunctionId, usize>) {
    info!("translating LIR program into Walrus module");
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
    module.globals.get_mut(sp_global_id).name = Some("global_sp".to_string());
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

    let mut mod_info = CodegenModuleInfo {
        mem_id,
        sp_global_id,
        fn_table_id,
        fn_table_slot_allocator,
        fn_table_allocated_slots: HashMap::new(),
        user_fn_ids: HashMap::new(),
        host_fn_ids: HashMap::new(),
    };

    // add host functions
    for host_fn in lir::host_function_references(lir) {
        let param_types: Vec<_> =
            host_fn.parameter_types.iter().copied().map(translate_val_type).collect();
        let return_types: Vec<_> =
            host_fn.return_type.iter().copied().map(translate_val_type).collect();
        let func_type = module.types.add(&param_types, &return_types);
        let (w_func_id, _) = module.add_import_func("env", host_fn.name, func_type);
        module.funcs.get_mut(w_func_id).name = Some(host_fn.name.to_string());
        trace!("added host function {:?} to function table", host_fn);
        mod_info.host_fn_ids.insert(host_fn, w_func_id);
    }

    let mut fn_infos = HashMap::new();

    // create empty functions in the walrus module. this needs to be done before
    // building the bodies so that the functions can reference each other
    for (lir_fid, func) in lir.user_functions.iter() {
        debug!("writing function setup for {:?}", lir_fid);
        let (wir_fid, fn_info) = write_function_setup(&mut module, func);
        fn_infos.insert(lir_fid, fn_info);
        mod_info.user_fn_ids.insert(lir_fid, wir_fid);

        // allocate slots in the function table for entrypoint functions
        if func.is_entrypoint {
            let slot = mod_info.fn_table_slot_allocator.allocate_slot();
            mod_info.fn_table_allocated_slots.insert(lir_fid, slot);
        }
    }

    // add user functions. these may allocate additional slots in the function
    // table for callbacks, every time the address of a function is taken.
    for (lir_fid, function) in lir.user_functions.iter() {
        debug!("adding user function {:?}", lir_fid);
        write_function_body(
            &mut mod_info,
            &mut module,
            lir_fid,
            function,
            fn_infos.get_mut(&lir_fid).unwrap(),
        );
    }

    // use element segments to install all functions for which fn table slots
    // were allocated
    for (lir_fn_id, slot) in &mod_info.fn_table_allocated_slots {
        let wir_fn_id = mod_info.user_fn_ids[lir_fn_id];
        module.elements.add(
            walrus::ElementKind::Active {
                table: mod_info.fn_table_id,
                offset: walrus::ConstExpr::Value(translate_usize(*slot)),
            },
            walrus::ElementItems::Functions(vec![wir_fn_id]),
        );
    }

    (module, mod_info.fn_table_allocated_slots)
}

struct CodegenFnInfo {
    /// Associates each LIR insn seq id with the walrus insn seq id.
    compound_labels: HashMap<lir::InsnSeqId, wir::InstrSeqId>,
    /// Associates each LIR value with the walrus local variable, if one
    /// exists.
    val_local_ids: HashMap<lir::ValRef, wir::LocalId>,
    /// Associates each LIR local variable the walrus local variable.
    var_local_ids: HashMap<lir::VarId, wir::LocalId>,
}

/// Context for codegen that applies to the entire function.
struct CodegenFnCtx<'b> {
    /// The function whose code is being generated.
    func: &'b lir::Function,
    /// The types of each LIR value
    types: &'b HashMap<lir::ValRef, lir::ValType>,
    /// Metadata for stackifying the function.
    stk: &'b stackify_lir::CfgStackification,
    /// The walrus local variable that holds the stack pointer.
    sp_local_id: Option<wir::LocalId>,
    /// The number of getters remaining for each LIR value.
    remaining_getters: HashMap<lir::ValRef, usize>,
}

fn write_function_setup(
    module: &mut walrus::Module,
    func: &lir::Function,
) -> (walrus::FunctionId, CodegenFnInfo) {
    // create the function builder
    let parameter_types: Vec<_> = func.local_vars[..func.num_parameters.into()]
        .iter()
        .copied()
        .map(translate_val_type)
        .collect();
    let return_types: Vec<_> =
        func.body.output_type.iter().copied().map(translate_val_type).collect();
    let mut function =
        walrus::FunctionBuilder::new(&mut module.types, &parameter_types, &return_types);
    if let Some(debug_fn_name) = &func.debug_fn_name {
        function.name(debug_fn_name.to_string());
    }

    let mut fn_info = CodegenFnInfo {
        compound_labels: HashMap::new(),
        val_local_ids: HashMap::new(),
        var_local_ids: HashMap::new(),
    };

    // allocate local variables for the function parameters
    let mut arg_locals = Vec::new();
    for (var_id, _) in func.local_vars[..func.num_parameters.into()].iter_enumerated() {
        trace!("allocating local variable for parameter {}", var_id);
        let wir_local_id = allocate_local_for_var(&mut module.locals, &mut fn_info, func, var_id);
        fn_info.var_local_ids.insert(var_id, wir_local_id);
        arg_locals.push(wir_local_id);
    }

    let wir_fid = function.finish(arg_locals, &mut module.funcs);

    (wir_fid, fn_info)
}

fn write_function_body<A: FnTableSlotAllocator>(
    mod_info: &mut CodegenModuleInfo<A>,
    module: &mut walrus::Module,
    fn_id: lir::FunctionId,
    func: &lir::Function,
    fn_info: &mut CodegenFnInfo,
) {
    trace!("writing function body for {:?}", func);

    // add stack pointer if needed
    let needs_stack_ptr = func.stack_space > 0;
    let sp_local_id: Option<walrus::LocalId> = needs_stack_ptr.then(|| {
        let local_id = module.locals.add(translate_val_type(lir::ValType::Ptr));
        module.locals.get_mut(local_id).name = Some("local_sp".to_string());
        local_id
    });

    // TODO(wishlist) we can make more efficient use of local variables by
    // having the ctx.uses become ctx.remaining_uses, which counts down each
    // time a getter for the value is taken. if a value is known to not be used
    // again, we can put the local variable into a pool of unused local
    // variables, which we'll pull from when we need a new local variable
    // without having to create a new one every time.

    let types = lir::infer_output_types(func);
    let stk = stackify_lir::stackify_cfg(func);

    // create the context for generating the function's code
    let mut ctx = CodegenFnCtx {
        func,
        types: &types,
        stk: &stk,
        sp_local_id,
        remaining_getters: stackify_lir::count_getters(&stk),
    };

    let mut insn_builder = module
        .funcs
        .get_mut(mod_info.user_fn_ids[&fn_id])
        .kind
        .unwrap_local_mut()
        .builder_mut()
        .func_body();

    // generate code, with or without a prologue/epilogue
    if let Some(stack_ptr_local) = sp_local_id {
        // there is a stack pointer. generate a prologue and epilogue that
        // initializes the stack pointer

        // subtract from the stack pointer
        insn_builder
            .global_get(mod_info.sp_global_id)
            .const_(translate_usize(func.stack_space))
            .binop(wir::BinaryOp::I32Sub)
            .local_tee(stack_ptr_local)
            .global_set(mod_info.sp_global_id);

        // put the function body in a block
        insn_builder.block(
            translate_instr_seq_type(
                &mut module.types,
                std::iter::empty(),
                func.body.output_type.iter().copied(),
            ),
            |inner_builder| {
                write_code(
                    &mut module.locals,
                    &mut module.types,
                    mod_info,
                    fn_info,
                    &mut ctx,
                    inner_builder,
                    func.body.body,
                );
            },
        );

        // add to the stack pointer
        insn_builder
            .local_get(stack_ptr_local)
            .const_(translate_usize(func.stack_space))
            .binop(wir::BinaryOp::I32Add)
            .global_set(mod_info.sp_global_id);
    } else {
        // there is no stack pointer, so no need for prologue or epilogue
        write_code(
            &mut module.locals,
            &mut module.types,
            mod_info,
            fn_info,
            &mut ctx,
            &mut insn_builder,
            func.body.body,
        );
    }
}

fn write_code<A: FnTableSlotAllocator>(
    // module: &mut walrus::Module,
    module_locals: &mut walrus::ModuleLocals,
    module_types: &mut walrus::ModuleTypes,
    mod_info: &mut CodegenModuleInfo<A>,
    fn_info: &mut CodegenFnInfo,
    ctx: &mut CodegenFnCtx,
    insn_builder: &mut walrus::InstrSeqBuilder,
    insn_seq_id: lir::InsnSeqId,
) {
    fn_info.compound_labels.insert(insn_seq_id, insn_builder.id());

    // track the operand stack. this is used so that we know what values we
    // are capturing (if they need to be stored in a local or just dropped),
    // as well as for validation
    let mut op_stack: Vec<ValRefOrStackPtr> = Vec::new();

    // the size of the prefix of the operand stack where all operands
    // needing saving are known to be saved
    let mut known_saved = 0;

    let seq_stk = &ctx.stk.seqs[&insn_seq_id];
    for (idx, insn) in ctx.func.insn_seqs[insn_seq_id].iter_enumerated() {
        trace!("writing code for instruction {:?}", insn);
        // let pc = lir::InsnPc(insn_seq_id, idx);
        let StackManipulators { captures, getters, inputs, outputs } = &seq_stk.manips[idx];

        // generate code to handle capturing and saving before this
        // instruction executes
        for _ in 0..*captures {
            let captured = op_stack.pop().expect("stackification should be correct");
            let ValRefOrStackPtr::ValRef(captured) = captured else {
                panic!("stackification should not cause stack ptr operand to be captured");
            };

            if !fn_info.val_local_ids.contains_key(&captured)
                && ctx.remaining_getters.contains_key(&captured)
            {
                let local_id =
                    allocate_local_for_val(module_locals, fn_info, ctx.types, ctx.func, captured);

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
                    (ctx.remaining_getters.contains_key(&v)
                        && !fn_info.val_local_ids.contains_key(&v))
                    .then_some(i)
                })
                .next_back(); // get the bottomost value needing saving
            // if required_pops is None, that means we did not find any
            // new values that needed to be saved
            if let Some(required_pops) = required_pops {
                // add instructions to pop values from the stack
                for i in 0..required_pops {
                    let local_id = allocate_local_for_val(
                        module_locals,
                        fn_info,
                        ctx.types,
                        ctx.func,
                        op_stack[op_stack.len() - i - 1].unwrap_val_ref(),
                    );
                    insn_builder.local_set(local_id);
                }
                // the bottommost value needing saving is now on top of the
                // stack
                let local_id = allocate_local_for_val(
                    module_locals,
                    fn_info,
                    ctx.types,
                    ctx.func,
                    op_stack[op_stack.len() - required_pops - 1].unwrap_val_ref(),
                );
                insn_builder.local_tee(local_id);
                // add instructions to push the saved values back onto the
                // stack
                for operand in op_stack.iter().skip(op_stack.len() - required_pops) {
                    insn_builder.local_get(fn_info.val_local_ids[&operand.unwrap_val_ref()]);
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
                    insn_builder.local_get(fn_info.val_local_ids[v]);
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
            I::LoopArg { initial_value: _ } => {
                // arguments do not show up in codegen. Unlike function
                // args, however, loop args do output values onto the
                // operand stack, so things are automatically handled.
            }
            I::Const(value) => {
                insn_builder.const_(translate_val(*value));
            }
            I::UserFunctionPtr { function } => {
                let slot = addr_of_function(mod_info, *function);
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
                insn_builder.load(mod_info.mem_id, load_kind, mem_arg);
            }
            I::MemStore { r#type, offset, ptr: _, value: _ } => {
                let store_kind = infer_store_kind(*r#type);
                let mem_arg = infer_mem_arg(*r#type, *offset);
                insn_builder.store(mod_info.mem_id, store_kind, mem_arg);
            }
            I::StackLoad { r#type, offset } => {
                let load_kind = infer_load_kind(*r#type);
                let mem_arg = infer_mem_arg(*r#type, *offset);
                insn_builder
                    .local_get(ctx.sp_local_id.expect("the presence of a StackAddr instruction means there must be a stack pointer local var"))
                    .load(mod_info.mem_id, load_kind, mem_arg);
            }
            I::StackStore { r#type, offset, value: _ } => {
                let store_kind = infer_store_kind(*r#type);
                let mem_arg = infer_mem_arg(*r#type, *offset);
                // stackification ensured that the stack pointer, followed
                // by the value, are already on the operand stack. we just
                // need to emit the store instruction.
                insn_builder.store(mod_info.mem_id, store_kind, mem_arg);
            }
            I::VarLoad { var_id } => {
                insn_builder.local_get(fn_info.var_local_ids[var_id]);
            }
            I::VarStore { var_id, value: _ } => {
                // allocate a local for the value, if one doesn't already
                // exist
                let wir_local_id =
                    fn_info.var_local_ids.get(var_id).copied().unwrap_or_else(|| {
                        allocate_local_for_var(module_locals, fn_info, ctx.func, *var_id)
                    });
                insn_builder.local_set(wir_local_id);
            }
            I::StackAddr { offset } => {
                insn_builder
                    .local_get(ctx.sp_local_id.expect("the presence of a StackAddr instruction means there must be a stack pointer local var"))
                    .const_(translate_usize(*offset))
                    .binop(wir::BinaryOp::I32Add);
            }
            I::CallHostFunction { function, args: _, output_type: _ } => {
                let callee = mod_info.lookup_host_fn(*function);
                insn_builder.call(callee);
            }
            I::CallUserFunction { function, args: _, output_type: _ } => {
                let callee = mod_info.user_fn_ids[function];
                insn_builder.call(callee);
            }
            I::CallIndirectFunction { function: _, args, output_type } => {
                let fn_table_id = mod_info.fn_table_id;
                let input_types: Vec<_> =
                    args.iter().map(|v| translate_val_type(ctx.types[v])).collect();
                let output_types: Vec<_> =
                    output_type.iter().copied().map(translate_val_type).collect();
                let type_id = module_types.add(&input_types, &output_types);
                insn_builder.call_indirect(type_id, fn_table_id);
            }
            I::UnaryOp { op, operand: _ } => {
                insn_builder.unop(translate_unary_op(*op));
            }
            I::BinaryOp { op, lhs, rhs } => {
                insn_builder.binop(translate_binary_op(*op, ctx.types[lhs], ctx.types[rhs]));
            }
            I::Break { target, values: _ } => {
                let target = fn_info.compound_labels[target];
                insn_builder.br(target);
            }
            I::ConditionalBreak { target, condition: _, values: _ } => {
                let target = fn_info.compound_labels[target];
                insn_builder.br_if(target);
            }
            I::Block(lir::Block { output_type, body }) => {
                let seq_type = translate_instr_seq_type(
                    module_types,
                    ctx.stk.seqs[body].inputs.iter().map(|v| ctx.types[&v.unwrap_val_ref()]),
                    output_type.iter().copied(),
                );

                insn_builder.block(seq_type, |inner_builder| {
                    write_code(
                        module_locals,
                        module_types,
                        mod_info,
                        fn_info,
                        ctx,
                        inner_builder,
                        *body,
                    );
                });
            }
            I::IfElse(lir::IfElse { condition: _, then_body, else_body, output_type }) => {
                let seq_type = translate_instr_seq_type(
                    module_types,
                    ctx.stk.seqs[then_body].inputs.iter().map(|v| ctx.types[&v.unwrap_val_ref()]),
                    output_type.iter().copied(),
                );

                // put the &mut in a RefCell so we can borrow it twice in
                // the two closures.
                let cell = RefCell::new((
                    &mut *ctx,
                    &mut *module_locals,
                    &mut *module_types,
                    &mut *mod_info,
                    &mut *fn_info,
                ));
                insn_builder.if_else(
                    seq_type,
                    |then_builder| {
                        // this may override the label in the other branch. this
                        // is okay as long as the label is set correctly while
                        // this branch is being built.
                        let (ctx, module_locals, module_types, mod_info, fn_info) =
                            &mut *cell.borrow_mut();
                        write_code(
                            module_locals,
                            module_types,
                            mod_info,
                            fn_info,
                            ctx,
                            then_builder,
                            *then_body,
                        );
                    },
                    |else_builder| {
                        // this may override the label in the other branch. this
                        // is okay as long as the label is set correctly while
                        // this branch is being built.
                        let (ctx, module_locals, module_types, mod_info, fn_info) =
                            &mut *cell.borrow_mut();
                        write_code(
                            module_locals,
                            module_types,
                            mod_info,
                            fn_info,
                            ctx,
                            else_builder,
                            *else_body,
                        );
                    },
                );
            }
            I::Loop(lir::Loop { body, inputs, output_type }) => {
                let seq_type = translate_instr_seq_type(
                    module_types,
                    inputs.iter().map(|v| ctx.types[v]),
                    output_type.iter().copied(),
                );

                insn_builder.loop_(seq_type, |inner_builder| {
                    write_code(
                        module_locals,
                        module_types,
                        mod_info,
                        fn_info,
                        ctx,
                        inner_builder,
                        *body,
                    );
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
        op_stack.extend(outputs);
    }

    // QUESTION should we deliberately skip stack manipulators at the end?
    // the end idx won't show up because it's not a real instruction
}
fn allocate_local_for_val(
    module_locals: &mut walrus::ModuleLocals,
    fn_info: &mut CodegenFnInfo,
    types: &HashMap<lir::ValRef, lir::ValType>,
    func: &lir::Function,
    val: lir::ValRef,
) -> wir::LocalId {
    let local_id = module_locals.add(translate_val_type(types[&val]));
    if let Some(name) = func.debug_val_names.get(&val) {
        module_locals.get_mut(local_id).name = Some(name.to_string());
    }
    fn_info.val_local_ids.insert(val, local_id);
    local_id
}
fn allocate_local_for_var(
    module_locals: &mut walrus::ModuleLocals,
    fn_info: &mut CodegenFnInfo,
    func: &lir::Function,
    var_id: lir::VarId,
) -> wir::LocalId {
    let local_id = module_locals.add(translate_val_type(func.local_vars[var_id]));
    if let Some(name) = func.debug_var_names.get(&var_id) {
        module_locals.get_mut(local_id).name = Some(name.to_string());
    }
    fn_info.var_local_ids.insert(var_id, local_id);
    local_id
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

fn infer_load_kind(r#type: lir::MemOpType) -> wir::LoadKind {
    use wir::ExtendedLoad as L;
    match r#type {
        lir::MemOpType::I8 => wir::LoadKind::I32_8 { kind: L::ZeroExtend },
        lir::MemOpType::I32 => wir::LoadKind::I32 { atomic: false },
        lir::MemOpType::I64 => wir::LoadKind::I64 { atomic: false },
        lir::MemOpType::F64 => wir::LoadKind::F64,
        lir::MemOpType::Ptr => wir::LoadKind::I32 { atomic: false },
        lir::MemOpType::FnPtr => wir::LoadKind::I32 { atomic: false },
    }
}

fn infer_store_kind(r#type: lir::MemOpType) -> wir::StoreKind {
    match r#type {
        lir::MemOpType::I8 => wir::StoreKind::I32_8 { atomic: false },
        lir::MemOpType::I32 => wir::StoreKind::I32 { atomic: false },
        lir::MemOpType::I64 => wir::StoreKind::I64 { atomic: false },
        lir::MemOpType::F64 => wir::StoreKind::F64,
        lir::MemOpType::Ptr => wir::StoreKind::I32 { atomic: false },
        lir::MemOpType::FnPtr => wir::StoreKind::I32 { atomic: false },
    }
}

fn infer_mem_arg(r#type: lir::MemOpType, offset: usize) -> wir::MemArg {
    let offset: u32 = offset.try_into().unwrap();
    match r#type {
        lir::MemOpType::I8 => wir::MemArg { align: 1, offset },
        lir::MemOpType::I32 => wir::MemArg { align: 4, offset },
        lir::MemOpType::I64 => wir::MemArg { align: 8, offset },
        lir::MemOpType::F64 => wir::MemArg { align: 8, offset },
        lir::MemOpType::Ptr => wir::MemArg { align: 4, offset },
        lir::MemOpType::FnPtr => wir::MemArg { align: 4, offset },
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
        (O::And, V::I32, V::I32) => Wo::I32And, // TODO: fix this
        (O::Or, V::I32, V::I32) => Wo::I32Or,   // TODO: fix this
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

#[cfg(test)]
mod tests {
    use lir::{lir_function, slotmap::SlotMap};
    use tracing::debug;

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
            fn func() -> [I32],
            vars: [],
            stack_space: 0,
            main: {
                break_(main)(constant(I32, 10));
            }
        }
        let mut lir = lir::Program::default();
        let mut functions: SlotMap<lir::FunctionId, ()> = SlotMap::with_key();
        let func_id = functions.insert(());
        lir.user_functions.insert(func_id, func);

        let (mut module, _) = lir_to_wasm(&lir, &mut TestFnTableSlotAllocator::default());
        let wasm = module.emit_wasm();
        std::fs::write("test.wasm", wasm).unwrap();
    }

    #[test]
    fn add_one() {
        lir_function! {
            fn add_one(I32 arg) -> [I32],
            vars: [],
            stack_space: 0,
            main: {
                [res] = IAdd(var_load(arg), constant(I32, 1));
                break_(main)(res);
            }
        }
        let mut lir = lir::Program::default();
        let mut functions: SlotMap<lir::FunctionId, ()> = SlotMap::with_key();
        let func_id = functions.insert(());
        lir.user_functions.insert(func_id, add_one);

        let (mut module, _) = lir_to_wasm(&lir, &mut TestFnTableSlotAllocator::default());
        let wasm = module.emit_wasm();
        std::fs::write("test.wasm", wasm).unwrap();
    }

    #[test]
    fn use_args_and_local_vars() {
        lir_function! {
            fn use_args(I32 a, I32 b) -> [I32],
            vars: [I32 c, I32 d],
            stack_space: 0,
            main: {
                [sum] = IAdd(var_load(a), var_load(b));
                var_store(d)(sum);
                var_store(c)(var_load(d));
            }
        }

        debug!("{:?}", use_args);

        let mut lir = lir::Program::default();
        let mut functions: SlotMap<lir::FunctionId, ()> = SlotMap::with_key();
        let func_id = functions.insert(());
        lir.user_functions.insert(func_id, use_args);

        let (mut module, _) = lir_to_wasm(&lir, &mut TestFnTableSlotAllocator::default());
        let wasm = module.emit_wasm();
        std::fs::write("test.wasm", wasm).unwrap();
    }

    #[test]
    fn use_stack() {
        lir_function! {
            fn my_function(I32 arg) -> [I32],
            vars: [],
            stack_space: 16,
            main: {
                stack_store(I32, 8)(var_load(arg));
                [_res] = stack_load(I32, 8);
                break_(main)(stack_addr(8));
            }
        }

        let mut lir = lir::Program::default();
        let mut functions: SlotMap<lir::FunctionId, ()> = SlotMap::with_key();
        let func_id = functions.insert(());
        lir.user_functions.insert(func_id, my_function);

        let (mut module, _) = lir_to_wasm(&lir, &mut TestFnTableSlotAllocator::default());
        let wasm = module.emit_wasm();
        std::fs::write("test.wasm", wasm).unwrap();
    }

    #[test]
    fn call_host_function() {
        let mut lir = lir::Program::default();
        let host_fn = lir::HostFunction(&lir::HostFunctionInfo {
            name: "defined_elsewhere",
            parameter_types: &[lir::ValType::I32, lir::ValType::F64],
            return_type: &[lir::ValType::F64, lir::ValType::I32],
        });

        lir_function! {
            fn my_function() -> [F64, I32],
            vars: [],
            stack_space: 0,
            main: {
                [a, b] = call_host_fn(host_fn -> [F64, I32])(
                    constant(I32, 10),
                    constant(F64, 20.0)
                );
                break_(main)(a, b);
            }
        }
        let mut functions: SlotMap<lir::FunctionId, ()> = SlotMap::with_key();
        let func_id = functions.insert(());
        lir.user_functions.insert(func_id, my_function);

        let (mut module, _) = lir_to_wasm(&lir, &mut TestFnTableSlotAllocator { next_slot: 0 });
        let wasm = module.emit_wasm();
        std::fs::write("test.wasm", wasm).unwrap();
    }

    #[test]
    fn call_user_function() {
        let mut functions: SlotMap<lir::FunctionId, ()> = SlotMap::with_key();

        let mut lir = lir::Program::default();
        lir_function! {
            fn callee(F64 arg) -> [I32],
            vars: [],
            stack_space: 0,
            main: {
                break_(main)(constant(I32, 10));
            }
        }
        let callee_id = functions.insert(());
        lir.user_functions.insert(callee_id, callee);
        lir_function! {
            fn caller() -> [I32],
            vars: [],
            stack_space: 0,
            main: {
                [a] = call_user_function(callee_id -> [I32])(constant(F64, 1.0));
                break_(main)(a);
            }
        }
        let caller_id = functions.insert(());
        lir.user_functions.insert(caller_id, caller);

        let (mut module, _) = lir_to_wasm(&lir, &mut TestFnTableSlotAllocator::default());
        let wasm = module.emit_wasm();
        std::fs::write("test.wasm", wasm).unwrap();
    }
}
