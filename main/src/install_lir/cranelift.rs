use std::{collections::HashMap, sync::Arc};

use crate::install_lir::Obj;

use cranelift_jit::{JITBuilder, JITModule};
use engine::{
    exec::jit::{InstallLir, InstallLirError},
    lir,
    sim::value::PackedAny,
    util::rng::CanonRng,
    workspace::Workspace,
};
use lir_to_cranelift::{
    cranelift_codegen::{self, settings},
    cranelift_module, lir_to_cranelift,
};

// TODO handle cleaning up old functions that are no longer needed

pub struct LirInstaller {
    this_isa: Arc<dyn cranelift_codegen::isa::TargetIsa>,
    module: cranelift_jit::JITModule,
}

impl Default for LirInstaller {
    fn default() -> Self {
        // The ISA and call conv code comes from the following links. In particular,
        // the forum post specifies that this is the correct way to get the calling
        // convention for the extern "C" ABI
        // https://users.rust-lang.org/t/calling-a-rust-function-from-cranelift/103948
        // https://github.com/bytecodealliance/cranelift-jit-demo/blob/main/src/jit.rs#L29-L39
        let this_isa = cranelift_native::builder()
            .expect("the selected target should be supported")
            .finish(settings::Flags::new(settings::builder())) // can change settings here to add optimizations e.g. leaf optimizations
            .expect("failed to finish ISA");

        let mut builder =
            JITBuilder::with_isa(this_isa.clone(), cranelift_module::default_libcall_names());

        // TODO add all host function symbols
        // builder.symbols(
        //     Self::HOST_FUNCTION_TABLE.all_host_functions().iter().map(|&hf| (hf.name, hf.addr)),
        // );

        let module = JITModule::new(builder);

        Self { this_isa, module }
    }
}

impl InstallLir for LirInstaller {
    type Obj = Obj;

    unsafe fn install_lir(&mut self, lir: &lir::Program) -> Result<Self::Obj, InstallLirError> {
        let lir_to_clm_fn_id = lir_to_cranelift(
            &mut self.module,
            lir,
            self.this_isa.triple(),
            |lir_fn_id, _, codegen_ctx| {
                std::fs::write(format!("{:?}.clif", lir_fn_id), codegen_ctx.func.to_string())
                    .unwrap();
            },
            |lir_fn_id, _, codegen_ctx| {
                let code =
                    codegen_ctx.compiled_code().expect("code should be compiled").code_buffer();
                std::fs::write(format!("{:?}.bin", lir_fn_id), code).unwrap();
            },
        );
        self.module.finalize_definitions().unwrap();

        let entrypoints = HashMap::from_iter(
            lir.user_functions
                .iter()
                .filter_map(
                    |(lir_fn_id, function)| {
                        if function.is_entrypoint { Some(lir_fn_id) } else { None }
                    },
                )
                .map(|lir_fn_id| {
                    // TODO for sanity, verify that each entrypoint has the
                    // signature specified in `JitEntrypoint`. However, Wasm
                    // should already catch if we indirectly call a function
                    // with the wrong signature.

                    let fn_ptr = self.module.get_finalized_function(lir_to_clm_fn_id[&lir_fn_id]);

                    // SAFETY: in the wasm32 target, a function pointer is
                    // represented by a i32 indicating the slot in the function
                    // table, so they literally have the same ABI
                    let fn_ptr = unsafe {
                        std::mem::transmute::<
                            *const u8,
                            unsafe extern "C" fn(
                                &mut Workspace,
                                &mut CanonRng,
                                *mut PackedAny,
                                u32,
                            ),
                        >(fn_ptr)
                    };
                    (lir_fn_id, fn_ptr)
                }),
        );
        Ok(Obj { entrypoints })
    }
}
