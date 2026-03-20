use std::collections::HashMap;

use crate::{
    exec::CanonExecutionContext,
    hir::{self, Expr},
    mir::{self, MirType, MirTypeInfo},
    sim::{observer::GlobalsSchema, patch::PatchSchema, turtle::TurtleSchema},
};

pub struct TypeMapping {
    globals_schema: GlobalsSchema,
    turtle_schema: TurtleSchema,
    patch_schema: PatchSchema,
    context_ty: MirType,
}

impl TypeMapping {
    pub fn new(
        globals_schema: GlobalsSchema,
        turtle_schema: TurtleSchema,
        patch_schema: PatchSchema,
    ) -> Self {
        let context_pointee_ty = CanonExecutionContext::mir_type_from_schemas(
            &globals_schema,
            &turtle_schema,
            &patch_schema,
        );
        let context_ptr_ty = MirTypeInfo::ptr_to(context_pointee_ty);
        Self { globals_schema, turtle_schema, patch_schema, context_ty: context_ptr_ty }
    }

    pub fn globals_schema(&self) -> &GlobalsSchema {
        &self.globals_schema
    }

    pub fn turtle_schema(&self) -> &TurtleSchema {
        &self.turtle_schema
    }

    pub fn patch_schema(&self) -> &PatchSchema {
        &self.patch_schema
    }

    pub fn context_ty(&self) -> MirType {
        self.context_ty.clone()
    }
}

pub struct HirToMirFnBuilder<'a, 'b> {
    pub hir: &'a hir::Program,
    pub type_mapping: &'a TypeMapping,
    pub mir: &'a mut mir::FunctionBuilder<'b>,
    pub translator: &'a mut HirToLirFnTranslator,
}

#[derive(Debug, Default)]
pub struct HirToLirFnTranslator {
    pub locals: HashMap<hir::LocalId, mir::LocalId>,
    /// Maps each HIR label to an MIR label, as well as the output local that
    /// breaks from that label should use.
    pub ctrl_flow_constructs: HashMap<hir::Label, (mir::Label, mir::LocalId)>,
    /// The local variable that contains the context parameter, if it exists.
    /// This should be automatically added if any expression in the function
    /// body needs a context parameter.
    pub context_param: Option<mir::LocalId>,
    /// The local variable that contains the self parameter, if it exists. This
    /// should automatically be added if any expression in the function body
    /// needs a self parameter.
    pub self_param: Option<SelfParam>,
}

#[derive(Debug)]
pub enum SelfParam {
    Turtle(mir::LocalId),
    Patch(mir::LocalId),
    Link(mir::LocalId),
    Any(mir::LocalId),
}

impl<'a, 'b> HirToMirFnBuilder<'a, 'b> {
    /// Generates MIR statements to evaluate the given HIR expression with all
    /// its dependencies. The return value is stored in a new out local whose id
    /// is returned. Any temporary values created during evaluation are stored
    /// in new local variables; this function does not reuse existing locals for
    /// temporaries.
    ///
    /// There is currently no checking if dependencies have already been
    /// evaluated.
    pub fn translate_expr(&mut self, expr: &hir::ExprKind) -> mir::TypedPlace {
        let output_ty = expr.output_type(self.hir).repr();
        // the expression's output will be stored in this local variable
        let (output_local, _output_local_decl) =
            self.mir.create_local(mir::LocalDecl { debug_name: None, ty: output_ty });
        // TODO could do something with the type assertion here
        expr.write_mir_execution(self, output_local);
        self.mir.typed_place(output_local)
    }

    // a bunch of boilerplate code to recreate the state of the builder. This is
    // only necessary because we want to keep all builder information packaged
    // together in a single struct. if we passed all the components of
    // HirToLirFnBuilder separately, this would not be necessary.
    pub fn with_inner_statement_seq<T>(
        &mut self,
        f: impl FnOnce(&mut HirToMirFnBuilder<'_, '_>) -> T,
    ) -> (Vec<mir::Statement>, T) {
        self.mir.with_inner_statement_seq(|lir| {
            let mut builder = HirToMirFnBuilder {
                hir: self.hir,
                type_mapping: self.type_mapping,
                mir: lir,
                translator: self.translator,
            };
            f(&mut builder)
        })
    }

    /// Returns the local variable that contains the context parameter, creating
    /// it as a function parameter if it does not exist.
    pub fn context_param(&mut self) -> mir::TypedPlace {
        let local_id = *self.translator.context_param.get_or_insert_with(|| {
            let (local_id, _local_decl) = self.mir.create_local(mir::LocalDecl {
                debug_name: Some("context".into()),
                ty: self.type_mapping.context_ty.clone(),
            });
            // TODO add as parameter to the function
            local_id
        });
        self.mir.typed_place(local_id)
    }
}

pub fn hir_to_mir(hir: &hir::Program, type_mapping: &TypeMapping) -> mir::Program {
    let mut builder = mir::ProgramBuilder::new();

    // iterate through each function and convert it to an MIR function
    for (hir_fn_id, hir_fn) in &hir.functions {
        // create a builder to track state while translating
        let mut mir_fn_builder = builder.create_function();
        let mut translator = HirToLirFnTranslator::default();

        let mut builder = HirToMirFnBuilder {
            hir,
            type_mapping,
            mir: &mut mir_fn_builder,
            translator: &mut translator,
        };

        // add all the nodes to the function body
        let (return_local, _) = builder.mir.create_local(mir::LocalDecl {
            debug_name: Some("return".into()),
            ty: hir_fn.body.output_type(hir).repr(),
        });
        hir_fn.body.write_mir_execution(&mut builder, return_local);

        mir_fn_builder.set_return(return_local);
        mir_fn_builder.finish();
    }

    todo!()
}
