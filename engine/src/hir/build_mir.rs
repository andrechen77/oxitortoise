use std::collections::BTreeMap;

use crate::{
    hir::{self, Expr, NameContext},
    mir,
};

mod type_mapping;

pub use type_mapping::{TypeMapping, make_type_mapping};

pub struct HirToMirFnBuilder<'a, 'b> {
    pub hir_names: NameContext<'a>,
    // pub hir: &'a hir::Program,
    pub type_mapping: &'a TypeMapping,
    pub mir: &'a mut mir::FunctionBuilder<'b>,
    pub translator: &'a mut HirToMirFnTranslator,
}

#[derive(Debug, Default)]
pub struct HirToMirFnTranslator {
    pub locals: BTreeMap<hir::LocalId, mir::LocalId>,
    /// Maps each HIR label to an MIR label, as well as the output local that
    /// breaks from that label should use. The output local might be none if
    /// it has not been used yet.
    pub ctrl_flow_constructs: BTreeMap<hir::Label, (mir::Label, Option<mir::LocalId>)>,
}

impl<'a, 'b> HirToMirFnBuilder<'a, 'b> {
    // a bunch of boilerplate code to recreate the state of the builder. This is
    // only necessary because we want to keep all builder information packaged
    // together in a single struct. if we passed all the components of
    // HirToLirFnBuilder separately, this would not be necessary.
    pub fn with_inner_statement_seq<T>(
        &mut self,
        f: impl FnOnce(&mut HirToMirFnBuilder) -> T,
    ) -> (Vec<mir::Statement>, T) {
        self.mir.with_inner_statement_seq(|mir| {
            let mut builder = HirToMirFnBuilder {
                hir_names: self.hir_names,
                type_mapping: self.type_mapping,
                mir,
                translator: self.translator,
            };
            f(&mut builder)
        })
    }

    pub fn with_locals<T>(
        &mut self,
        local_vars: &BTreeMap<hir::LocalId, hir::LocalDecl>,
        f: impl FnOnce(&mut HirToMirFnBuilder) -> T,
    ) -> T {
        f(&mut HirToMirFnBuilder {
            hir_names: self.hir_names.with_locals(local_vars),
            type_mapping: self.type_mapping,
            mir: self.mir,
            translator: self.translator,
        })
    }
}

pub fn hir_to_mir(hir: &hir::Program) -> mir::Program {
    let type_mapping = make_type_mapping(hir);

    let mut builder = mir::ProgramBuilder::new();

    // iterate through each function and convert it to an MIR function
    for (hir_fn_id, hir_fn) in &hir.functions {
        let hir_fn_body = &hir.function_bodies[hir_fn_id];

        // create a builder to track state while translating
        let mut mir_fn_builder = builder.create_function();
        let mut translator = HirToMirFnTranslator::default();

        let mut builder = HirToMirFnBuilder {
            hir_names: NameContext::from_program(hir),
            type_mapping: &type_mapping,
            mir: &mut mir_fn_builder,
            translator: &mut translator,
        };

        let hir::Function { debug_name, parameters, return_ty, is_entrypoint } = hir_fn;

        // add all the function parameters to the function builder
        for (hir_param_id, param_decl) in parameters {
            let ty = builder.type_mapping.local_var_ty(*hir_param_id);
            let mir_param_id = builder.mir.create_parameter(mir::LocalDecl {
                debug_name: Some(param_decl.debug_name.clone()),
                ty,
            });
            builder.translator.locals.insert(*hir_param_id, mir_param_id);
        }

        // add all the nodes to the function body
        let return_value = builder
            .with_locals(&hir_fn.parameters, |builder| hir_fn_body.write_mir_execution(builder));

        if let Some(return_value) = return_value {
            mir_fn_builder.set_return(return_value);
        }
        mir_fn_builder.finish();
    }

    todo!()
}
