use cranelift_codegen::{ir as cl, isa};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use target_lexicon::Triple;

pub extern crate lir;

pub fn lir_to_cranelift(lir: &lir::Program, triple: &Triple) {
    // // The ISA and call conv code comes from the following links. In particular,
    // // the forum post specifies that this is the correct way to get the calling
    // // convention for the extern "C" ABI
    // // https://users.rust-lang.org/t/calling-a-rust-function-from-cranelift/103948
    // // https://github.com/bytecodealliance/cranelift-jit-demo/blob/main/src/jit.rs#L29-L39
    // let isa = cranelift_native::builder()
    //     .expect("the selected target should be supported")
    //     .finish(settings::Flags::new(settings::builder()))
    //     .expect("failed to finish ISA");
    // let call_conv = isa.default_call_conv();

    // let module =
    //     JITModule::new(JITBuilder::with_isa(isa, cranelift_module::default_libcall_names()));

    let lir::Program { user_functions } = lir;

    let mut ctx = FunctionBuilderContext::new();

    for (lir_fn_id, func) in user_functions.iter() {
        translate_function(&mut ctx, func, triple);
    }
}

fn translate_function(
    ctx: &mut FunctionBuilderContext,
    func: &lir::Function,
    triple: &Triple,
) -> cl::Function {
    let lir::Function {
        local_vars,
        num_parameters,
        body,
        insn_seqs,
        stack_space,
        debug_fn_name,
        debug_val_names,
        debug_var_names,
        is_entrypoint,
    } = func;

    // stealing from https://docs.rs/cranelift-frontend/latest/cranelift_frontend/

    // The forum post specifies that this is the correct way to get the calling
    // convention for the extern "C" ABI
    // https://users.rust-lang.org/t/calling-a-rust-function-from-cranelift/103948
    let call_conv = isa::CallConv::triple_default(triple);
    let signature = cl::Signature {
        params: (0..*num_parameters)
            .map(|i| translate_val_type(local_vars[lir::VarId(i)], triple))
            .map(|t| cl::AbiParam::new(t))
            .collect(),
        returns: body
            .output_type
            .iter()
            .map(|&t| translate_val_type(t, triple))
            .map(|t| cl::AbiParam::new(t))
            .collect(),
        call_conv,
    };
    // TODO put something meanigful in these integers
    let name = cl::UserFuncName::user(0, 0);

    let mut cl_func = cl::Function::with_name_signature(name, signature);

    let mut builder = FunctionBuilder::new(&mut cl_func, ctx);

    // TODO write function body
    cl_func
}

fn translate_val_type(val_type: lir::ValType, triple: &Triple) -> cl::Type {
    match val_type {
        lir::ValType::I32 => cl::types::I32,
        lir::ValType::I64 => cl::types::I64,
        lir::ValType::F64 => cl::types::F64,
        lir::ValType::Ptr => cl::Type::triple_pointer_type(triple),
        lir::ValType::FnPtr => cl::Type::triple_pointer_type(triple),
    }
}
