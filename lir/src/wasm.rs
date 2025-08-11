use crate::lir;

pub fn lir_to_wasm(lir: &lir::Program) -> walrus::Module {
    let config = walrus::ModuleConfig::default();

    let mut module = walrus::Module::with_config(config);

    // for function in &lir.functions {}

    todo!()
}

pub fn add_function(module: &mut walrus::Module, lir: &lir::Function) {
    let parameter_types: Vec<_> = lir
        .parameter_types
        .iter()
        .copied()
        .map(translate_val_type)
        .collect();
    let return_types: Vec<_> = lir
        .return_types
        .iter()
        .copied()
        .map(translate_val_type)
        .collect();
    let mut function =
        walrus::FunctionBuilder::new(&mut module.types, &parameter_types, &return_types);
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
