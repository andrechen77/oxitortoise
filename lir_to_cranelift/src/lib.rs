use std::collections::HashMap;

// `lir` and `clir` prefixes are used to easily distinguish between parallel
// concepts in the two IRs

use cranelift_codegen::{
    ir::{self as clir, InstBuilder},
    isa,
};
// prefixed so that we know what is part of the Cranelift frontend API rather
// than our own builders
use cranelift_frontend as clf;
// prefixed so that we know what is part of the high level Cranelift module API
// rather than the core Cranelift IR
use cranelift_module as clm;
use either::Either;
use lir::smallvec::{SmallVec, smallvec};
use target_lexicon::Triple;

pub extern crate lir;

pub fn lir_to_cranelift(
    module: &mut impl clm::Module,
    lir: &lir::Program,
    triple: &Triple,
) -> HashMap<lir::FunctionId, clm::FuncId> {
    let lir::Program { user_functions } = lir;

    // make an initial pass over the functions to declare them in the module
    let mut lir_to_clm_fn_id = HashMap::new();
    for (lir_fn_id, lir_fn) in user_functions.iter() {
        // the different function ids ensure that names never collide
        let unique_name =
            format!("{:?} {}", lir_fn_id, lir_fn.debug_fn_name.as_deref().unwrap_or("unnamed"));
        let parameter_types = &lir_fn.local_vars[..lir_fn.num_parameters.into()].raw;
        let signature = translate_fn_signature(parameter_types, &lir_fn.body.output_type, triple);
        let clm_fn_id = module
            .declare_function(
                &unique_name,
                // local even when the function is an entrypoint because we will
                // be calling it via function pointer
                clm::Linkage::Local,
                &signature,
            )
            .expect("failed to declare function");
        let is_entrypoint = lir_fn.is_entrypoint; // TODO do something with this
        lir_to_clm_fn_id.insert(lir_fn_id, clm_fn_id);
    }

    let mut codegen_ctx = module.make_context();
    let mut clf_fn_builder_ctx = clf::FunctionBuilderContext::new();

    // go through each function and translate the body
    for (lir_fn_id, lir_fn) in user_functions.iter() {
        let mut use_host_fn = |lir_fn: Either<lir::HostFunction, lir::FunctionId>,
                               cl_fn: &mut clir::Function| {
            let clm_fn_id = match lir_fn {
                Either::Left(host_fn) => {
                    let host_fn_signature = translate_fn_signature(
                        host_fn.parameter_types,
                        host_fn.return_type,
                        triple,
                    );
                    module
                        .declare_function(host_fn.name, clm::Linkage::Import, &host_fn_signature)
                        .expect("host functions should always succeed in being declared")
                }
                Either::Right(user_fn_id) => lir_to_clm_fn_id[&user_fn_id],
            };
            module.declare_func_in_func(clm_fn_id, cl_fn)
        };
        translate_function(
            &mut codegen_ctx.func,
            &mut clf_fn_builder_ctx,
            lir_fn,
            triple,
            &mut use_host_fn,
        );

        module.define_function(lir_to_clm_fn_id[&lir_fn_id], &mut codegen_ctx).unwrap();

        println!("codegen_ctx: {}", codegen_ctx.func);

        // cleanup for the next function to reuse the same context
        module.clear_context(&mut codegen_ctx);
    }

    lir_to_clm_fn_id
}

/// Writes the provided LIR function into `out_function` using
/// `clf_fn_builder_ctx`. `add_fn` is a callback that creates an internal
/// `FuncRef` for previously unencountered host or user functions.
///
/// Note that this function intentionally does not deal with `cranelift-module`.
/// Its only purpose is to write function bodies
fn translate_function(
    out_function: &mut clir::Function,
    clf_fn_builder_ctx: &mut clf::FunctionBuilderContext,
    lir: &lir::Function,
    triple: &Triple,
    add_fn: &mut impl FnMut(
        Either<lir::HostFunction, lir::FunctionId>,
        &mut clir::Function,
    ) -> clir::FuncRef,
) {
    let mut clf_builder = clf::FunctionBuilder::new(out_function, clf_fn_builder_ctx);

    // The forum post specifies that this is the correct way to get the calling
    // convention for the extern "C" ABI
    // https://users.rust-lang.org/t/calling-a-rust-function-from-cranelift/103948
    let call_conv = isa::CallConv::triple_default(triple);
    let signature = clir::Signature {
        params: (0..lir.num_parameters)
            .map(|i| translate_val_type(lir.local_vars[lir::VarId(i)], triple))
            .map(|t| clir::AbiParam::new(t))
            .collect(),
        returns: lir
            .body
            .output_type
            .iter()
            .map(|&t| translate_val_type(t, triple))
            .map(|t| clir::AbiParam::new(t))
            .collect(),
        call_conv,
    };
    clf_builder.func.signature = signature;

    // TODO write function body

    // we currently do not use the stack for anything
    assert_eq!(lir.stack_space, 0);

    let lir_types = lir::infer_output_types(lir);

    let mut builder = FunctionBuilder {
        cl: &mut clf_builder,
        lir_function: lir,
        lir_types: &lir_types,
        host_fn_imports: HashMap::new(),
        user_fn_imports: HashMap::new(),
        value_map: HashMap::new(),
        var_map: HashMap::new(),
        break_targets: HashMap::new(),
    };

    // declare all local variables
    for (lir_var_id, var_ty) in lir.local_vars.iter_enumerated() {
        let cl_var_id = builder.cl.declare_var(translate_val_type(*var_ty, triple));
        builder.var_map.insert(lir_var_id, cl_var_id);
    }

    // set up entry block
    let entry_bb = builder.cl.create_block();
    builder.cl.append_block_params_for_function_params(entry_bb);
    builder.cl.seal_block(entry_bb);
    builder.cl.switch_to_block(entry_bb);

    // make parameters available as cranelift variables
    let lir_parameters = lir.local_vars[..lir.num_parameters.into()].keys();
    let cl_parameters =
        builder.cl.block_params(entry_bb).iter().copied().collect::<Vec<_>>().into_iter();
    for (lir_var_id, cl_val) in lir_parameters.zip(cl_parameters) {
        builder.cl.def_var(builder.var_map[&lir_var_id], cl_val);
    }

    // write the instructions
    let return_values = translate_insn_seq_with_end_break(
        &mut builder,
        &lir.body.output_type,
        [lir.body.body],
        triple,
        |builder| {
            translate_insn_seq(builder, lir.body.body, triple, add_fn);
        },
    );
    builder.cl.ins().return_(&return_values);

    // we are done building the function
    clf_builder.finalize();
}

struct FunctionBuilder<'a, 'b> {
    cl: &'a mut clf::FunctionBuilder<'b>,
    lir_function: &'a lir::Function,
    lir_types: &'a HashMap<lir::ValRef, lir::ValType>,
    /// Maps each LIR host fn reference to a Cranelift function reference
    host_fn_imports: HashMap<lir::HostFunction, clir::FuncRef>,
    /// Maps each LIR user function reference to a Cranelift function reference
    user_fn_imports: HashMap<lir::FunctionId, clir::FuncRef>,
    /// Maps each LIR value reference to a Cranelift value
    value_map: HashMap<lir::ValRef, clir::Value>,
    /// Maps each LIR variable id to a Cranelift variable.
    var_map: HashMap<lir::VarId, clf::Variable>,
    /// Maps each instruction sequence id to the block that control should jump
    /// to when an LIR break instruction targets that instruction sequence id
    break_targets: HashMap<lir::InsnSeqId, clir::Block>,
}

fn translate_insn_seq(
    builder: &mut FunctionBuilder,
    insn_seq_id: lir::InsnSeqId,
    triple: &Triple,
    add_fn: &mut impl FnMut(
        Either<lir::HostFunction, lir::FunctionId>,
        &mut clir::Function,
    ) -> clir::FuncRef,
) {
    for (insn_idx, insn) in builder.lir_function.insn_seqs[insn_seq_id].iter_enumerated() {
        let lir_pc = lir::InsnPc(insn_seq_id, insn_idx);
        let cl_values: SmallVec<[clir::Value; 1]> = match insn {
            lir::InsnKind::Break { target, values } => {
                let dst_bb = builder.break_targets[target];
                let values: Vec<_> = values
                    .iter()
                    .map(|arg| clir::BlockArg::Value(builder.value_map[arg]))
                    .collect();
                let insn_ref = builder.cl.ins().jump(dst_bb, &values);
                builder.cl.inst_results(insn_ref).iter().copied().collect()
            }
            lir::InsnKind::ConditionalBreak { .. } => {
                unimplemented!("idt this instruction is currently used")
            }
            lir::InsnKind::Block(lir::Block { output_type, body }) => {
                translate_insn_seq_with_end_break(
                    builder,
                    output_type,
                    [*body],
                    triple,
                    |builder| {
                        translate_insn_seq(builder, *body, triple, add_fn);
                    },
                )
            }
            lir::InsnKind::IfElse(lir::IfElse { output_type, condition, then_body, else_body }) => {
                // create a BB for each branch
                let then_bb = builder.cl.create_block();
                let else_bb = builder.cl.create_block();

                // add the branch instruction
                let condition = builder.value_map[condition];
                builder.cl.ins().brif(condition, then_bb, &[], else_bb, &[]);
                builder.cl.seal_block(then_bb);
                builder.cl.seal_block(else_bb);

                translate_insn_seq_with_end_break(
                    builder,
                    output_type,
                    [*then_body, *else_body],
                    triple,
                    |builder| {
                        // add instructions to each branch
                        builder.cl.switch_to_block(then_bb);
                        translate_insn_seq(builder, *then_body, triple, add_fn);
                        builder.cl.switch_to_block(else_bb);
                        translate_insn_seq(builder, *else_body, triple, add_fn);
                    },
                )
            }
            lir::InsnKind::Loop(lir::Loop { .. }) => {
                todo!("idt this instruction is currently used")
            }
            lir::InsnKind::LoopArg { .. } => {
                unimplemented!("idt this instruction is currently used")
            }
            lir::InsnKind::CallHostFunction { function, output_type: _, args } => {
                let func_ref = *builder
                    .host_fn_imports
                    .entry(*function)
                    .or_insert_with(|| add_fn(Either::Left(*function), builder.cl.func));
                let args: Vec<_> = args.iter().map(|arg| builder.value_map[arg]).collect();
                let insn_ref = builder.cl.ins().call(func_ref, &args);
                builder.cl.inst_results(insn_ref).iter().copied().collect()
            }
            lir::InsnKind::CallUserFunction { function, output_type: _, args } => {
                let func_ref = *builder
                    .user_fn_imports
                    .entry(*function)
                    .or_insert_with(|| add_fn(Either::Right(*function), builder.cl.func));
                let args: Vec<_> = args.iter().map(|arg| builder.value_map[arg]).collect();
                let insn_ref = builder.cl.ins().call(func_ref, &args);
                builder.cl.inst_results(insn_ref).iter().copied().collect()
            }
            lir::InsnKind::CallIndirectFunction { function, output_type, args } => {
                let callee = builder.value_map[function];
                let arg_types: Vec<_> = args.iter().map(|arg| builder.lir_types[arg]).collect();
                let signature = translate_fn_signature(&arg_types, output_type, triple);
                let sig_ref = builder.cl.import_signature(signature);
                let args: Vec<_> = args.iter().map(|arg| builder.value_map[arg]).collect();
                let insn_ref = builder.cl.ins().call_indirect(sig_ref, callee, &args);
                builder.cl.inst_results(insn_ref).iter().copied().collect()
            }
            lir::InsnKind::UserFunctionPtr { function } => {
                let func_ref = *builder
                    .user_fn_imports
                    .entry(*function)
                    .or_insert_with(|| add_fn(Either::Right(*function), builder.cl.func));
                let val = builder
                    .cl
                    .ins()
                    .func_addr(translate_val_type(lir::ValType::FnPtr, triple), func_ref);
                smallvec![val]
            }
            lir::InsnKind::Const(val) => {
                let val = match *val {
                    lir::Value::I32(val) => builder.cl.ins().iconst(clir::types::I32, val as i64),
                    lir::Value::I64(val) => builder.cl.ins().iconst(clir::types::I64, val as i64),
                    lir::Value::F64(val) => builder.cl.ins().f64const(val),
                    lir::Value::Ptr(val) => unimplemented!("cannot embed pointers into consts"),
                    lir::Value::FnPtr(val) => {
                        unimplemented!("cannot embed function pointers into consts")
                    }
                };
                smallvec![val]
            }
            lir::InsnKind::BinaryOp { op, lhs, rhs } => {
                let lhs = builder.value_map[lhs];
                let rhs = builder.value_map[rhs];
                use lir::BinaryOpcode as B;
                let val = match op {
                    B::IAdd => builder.cl.ins().iadd(lhs, rhs),
                    B::ISub => builder.cl.ins().isub(lhs, rhs),
                    B::IMul => builder.cl.ins().imul(lhs, rhs),
                    B::FAdd => builder.cl.ins().fadd(lhs, rhs),
                    B::FSub => builder.cl.ins().fsub(lhs, rhs),
                    B::FMul => builder.cl.ins().fmul(lhs, rhs),
                    B::FDiv => builder.cl.ins().fdiv(lhs, rhs),
                    B::And => builder.cl.ins().band(lhs, rhs),
                    B::Or => builder.cl.ins().bor(lhs, rhs),
                    B::SLt | B::SGt | B::ULt | B::UGt | B::IEq | B::INeq => {
                        let cond = match op {
                            B::SLt => clir::condcodes::IntCC::SignedLessThan,
                            B::SGt => clir::condcodes::IntCC::SignedGreaterThan,
                            B::ULt => clir::condcodes::IntCC::UnsignedLessThan,
                            B::UGt => clir::condcodes::IntCC::UnsignedGreaterThan,
                            B::IEq => clir::condcodes::IntCC::Equal,
                            B::INeq => clir::condcodes::IntCC::NotEqual,
                            _ => unreachable!(),
                        };
                        builder.cl.ins().icmp(cond, lhs, rhs)
                    }
                    B::FLt | B::FLte | B::FGt | B::FGte | B::FEq => {
                        let cond = match op {
                            B::FLt => clir::condcodes::FloatCC::LessThan,
                            B::FLte => clir::condcodes::FloatCC::LessThanOrEqual,
                            B::FGt => clir::condcodes::FloatCC::GreaterThan,
                            B::FGte => clir::condcodes::FloatCC::GreaterThanOrEqual,
                            B::FEq => clir::condcodes::FloatCC::Equal,
                            _ => unreachable!(),
                        };
                        builder.cl.ins().fcmp(cond, lhs, rhs)
                    }
                };
                smallvec![val]
            }
            lir::InsnKind::UnaryOp { op, operand } => {
                let operand = builder.value_map[operand];
                let val = match op {
                    lir::UnaryOpcode::FNeg => builder.cl.ins().fneg(operand),
                    lir::UnaryOpcode::Not => builder.cl.ins().bnot(operand),
                    lir::UnaryOpcode::I64ToI32 => {
                        builder.cl.ins().ireduce(clir::types::I32, operand)
                    }
                };
                smallvec![val]
            }
            lir::InsnKind::DeriveField { offset, ptr } => {
                let ptr = builder.value_map[ptr];
                let val = builder.cl.ins().iadd_imm(ptr, i64::try_from(*offset).unwrap());
                smallvec![val]
            }
            lir::InsnKind::DeriveElement { element_size, ptr, index } => {
                let ptr = builder.value_map[ptr];
                let index = builder.value_map[index];
                let offset =
                    builder.cl.ins().imul_imm(index, i64::try_from(*element_size).unwrap());
                let val = builder.cl.ins().iadd(ptr, offset);
                smallvec![val]
            }
            lir::InsnKind::MemLoad { r#type, offset, ptr } => {
                let ptr = builder.value_map[ptr];
                let load_ty = translate_mem_op_type(*r#type, triple);
                let val = builder.cl.ins().load(
                    load_ty,
                    clir::MemFlags::new(),
                    ptr,
                    i32::try_from(*offset).unwrap(),
                );
                smallvec![val]
            }
            lir::InsnKind::MemStore { r#type: _, offset, ptr, value } => {
                let ptr = builder.value_map[ptr];
                let value = builder.value_map[value];
                builder.cl.ins().store(
                    clir::MemFlags::new(),
                    value,
                    ptr,
                    i32::try_from(*offset).unwrap(),
                );
                smallvec![]
            }
            lir::InsnKind::StackLoad { .. } => {
                unimplemented!("currently unused")
            }
            lir::InsnKind::StackStore { .. } => {
                unimplemented!("currently unused")
            }
            lir::InsnKind::StackAddr { .. } => {
                unimplemented!("currently unused")
            }
            lir::InsnKind::VarLoad { var_id } => {
                let var = builder.var_map[var_id];
                let val = builder.cl.use_var(var);
                smallvec![val]
            }
            lir::InsnKind::VarStore { var_id, value } => {
                let var = builder.var_map[var_id];
                let value = builder.value_map[value];
                builder.cl.def_var(var, value);
                smallvec![]
            }
        };
        // make the values available for later instructions
        for (i, cl_value) in cl_values.iter().enumerate() {
            let val_ref = lir::ValRef(lir_pc, u8::try_from(i).unwrap());
            builder.value_map.insert(val_ref, *cl_value);
        }
    }
}

/// Translates a LIR instruction sequence such that any breaks targeting the
/// specified instruction sequence will jump to the end of the instruction sequence.
/// Returns the Cranelift SSA values of the results of breaking to that instruction sequence.
fn translate_insn_seq_with_end_break<const N: usize>(
    builder: &mut FunctionBuilder,
    break_values: &[lir::ValType],
    targeted_insn_seq_ids: [lir::InsnSeqId; N],
    triple: &Triple,
    add_instructions: impl FnOnce(&mut FunctionBuilder),
) -> SmallVec<[clir::Value; 1]> {
    // create a new BB for any breaks that target the instruction sequence
    let break_bb = builder.cl.create_block();
    for val_type in break_values {
        builder.cl.append_block_param(break_bb, translate_val_type(*val_type, triple));
    }
    for targeted_insn_seq_id in targeted_insn_seq_ids {
        builder.break_targets.insert(targeted_insn_seq_id, break_bb);
    }

    // keep adding instructions to the same BB; there's no
    // need to switch until we actually encounter a branch
    add_instructions(builder);

    // now that all the instructions in the LIR block are added,
    // switch to the new BB to keep adding instructions
    builder.cl.switch_to_block(break_bb);
    builder.cl.seal_block(break_bb);

    // make the output of the LIR block available in builder.value_map
    builder.cl.block_params(break_bb).iter().copied().collect()
}

fn translate_val_type(val_type: lir::ValType, triple: &Triple) -> clir::Type {
    match val_type {
        lir::ValType::I8 => clir::types::I8,
        lir::ValType::I32 => clir::types::I32,
        lir::ValType::I64 => clir::types::I64,
        lir::ValType::F64 => clir::types::F64,
        lir::ValType::Ptr => clir::Type::triple_pointer_type(triple),
        lir::ValType::FnPtr => clir::Type::triple_pointer_type(triple),
    }
}

fn translate_mem_op_type(mem_op_type: lir::ValType, triple: &Triple) -> clir::Type {
    match mem_op_type {
        lir::ValType::I8 => clir::types::I8,
        lir::ValType::I32 => clir::types::I32,
        lir::ValType::I64 => clir::types::I64,
        lir::ValType::F64 => clir::types::F64,
        lir::ValType::Ptr => clir::Type::triple_pointer_type(triple),
        lir::ValType::FnPtr => clir::Type::triple_pointer_type(triple),
    }
}

fn translate_fn_signature(
    params: &[lir::ValType],
    returns: &[lir::ValType],
    triple: &Triple,
) -> clir::Signature {
    let params = params
        .iter()
        .map(|param| translate_val_type(*param, triple))
        .map(|t| clir::AbiParam::new(t))
        .collect();
    let returns = returns
        .iter()
        .map(|ret| translate_val_type(*ret, triple))
        .map(|t| clir::AbiParam::new(t))
        .collect();
    let call_conv = isa::CallConv::triple_default(triple);
    clir::Signature { params, returns, call_conv }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use cranelift_codegen::settings;
    use cranelift_jit::{JITBuilder, JITModule};
    use lir::{lir_function, slotmap::SlotMap};

    use super::*;

    fn create_isa_and_module() -> (Arc<dyn isa::TargetIsa>, JITModule) {
        // The ISA and call conv code comes from the following links. In particular,
        // the forum post specifies that this is the correct way to get the calling
        // convention for the extern "C" ABI
        // https://users.rust-lang.org/t/calling-a-rust-function-from-cranelift/103948
        // https://github.com/bytecodealliance/cranelift-jit-demo/blob/main/src/jit.rs#L29-L39
        let isa = cranelift_native::builder()
            .expect("the selected target should be supported")
            .finish(settings::Flags::new(settings::builder()))
            .expect("failed to finish ISA");

        let module = JITModule::new(JITBuilder::with_isa(
            isa.clone(),
            cranelift_module::default_libcall_names(),
        ));

        (isa, module)
    }

    #[test]
    fn test_return_const() {
        let mut program = lir::Program::default();
        let mut functions = SlotMap::with_key();
        let key = functions.insert(());
        lir_function! {
            fn return_const() -> [I32],
            vars: [],
            stack_space: 0,
            main: {
                break_(main)(constant(I32, 10));
            }
        }
        program.user_functions.insert(key, return_const);

        let (isa, mut module) = create_isa_and_module();
        let lir_to_clm_fn_id = lir_to_cranelift(&mut module, &program, isa.triple());

        module.finalize_definitions().unwrap();

        let fn_ptr = module.get_finalized_function(lir_to_clm_fn_id[&key]);
        let fn_ptr: extern "C" fn() -> i32 = unsafe { std::mem::transmute(fn_ptr) };
        assert_eq!(fn_ptr(), 10);
    }

    #[test]
    fn test_branch_with_params() {
        let mut program = lir::Program::default();
        let mut functions = SlotMap::with_key();
        let key = functions.insert(());
        lir_function! {
            fn find_min(I32 a, I32 b) -> [I32],
            vars: [],
            stack_space: 0,
            main: {
                [a] = var_load(a);
                [b] = var_load(b);
                [] = if_else(-> [])(SLt(a, b)) then: {
                    break_(main)(a);
                } else_: {
                    break_(main)(b);
                };
            }
        }
        program.user_functions.insert(key, find_min);

        let (isa, mut module) = create_isa_and_module();
        let lir_to_clm_fn_id = lir_to_cranelift(&mut module, &program, isa.triple());

        module.finalize_definitions().unwrap();

        let fn_ptr = module.get_finalized_function(lir_to_clm_fn_id[&key]);
        let fn_ptr: extern "C" fn(i32, i32) -> i32 = unsafe { std::mem::transmute(fn_ptr) };
        assert_eq!(fn_ptr(10, 20), 10);
        assert_eq!(fn_ptr(20, 10), 10);
    }

    #[test]
    fn test_mutual_recursion() {
        let mut program = lir::Program::default();
        let mut functions = SlotMap::with_key();

        let is_even_id = functions.insert(());
        let is_odd_id = functions.insert(());

        lir_function! {
            fn is_even(I32 n) -> [I32],
            vars: [],
            stack_space: 0,
            main: {
                [n_val] = var_load(n);
                [] = if_else(-> [])(IEq(n_val, constant(I32, 0))) then: {
                    break_(main)(constant(I32, 1));
                } else_: {
                    [n_minus_one] = ISub(n_val, constant(I32, 1));
                    [odd_val] = call_user_function(is_odd_id -> [I32])(n_minus_one);
                    break_(main)(odd_val);
                };
            }
        }
        lir_function! {
            fn is_odd(I32 n) -> [I32],
            vars: [],
            stack_space: 0,
            main: {
                [n_val] = var_load(n);
                [] = if_else(-> [])(IEq(n_val, constant(I32, 0))) then: {
                    break_(main)(constant(I32, 0));
                } else_: {
                    [n_minus_one] = ISub(n_val, constant(I32, 1));
                    [even_val] = call_user_function(is_even_id -> [I32])(n_minus_one);
                    break_(main)(even_val);
                };
            }
        }

        program.user_functions.insert(is_even_id, is_even);
        program.user_functions.insert(is_odd_id, is_odd);

        let (isa, mut module) = create_isa_and_module();
        let lir_to_clm_fn_id = lir_to_cranelift(&mut module, &program, isa.triple());

        module.finalize_definitions().unwrap();

        let is_even_ptr = module.get_finalized_function(lir_to_clm_fn_id[&is_even_id]);
        let is_even_fn: extern "C" fn(i32) -> i32 = unsafe { std::mem::transmute(is_even_ptr) };

        let is_odd_ptr = module.get_finalized_function(lir_to_clm_fn_id[&is_odd_id]);
        let is_odd_fn: extern "C" fn(i32) -> i32 = unsafe { std::mem::transmute(is_odd_ptr) };

        // Test several values
        for i in 0..10 {
            assert_eq!(is_even_fn(i), if i % 2 == 0 { 1 } else { 0 });
            assert_eq!(is_odd_fn(i), if i % 2 == 1 { 1 } else { 0 });
        }
    }
}
