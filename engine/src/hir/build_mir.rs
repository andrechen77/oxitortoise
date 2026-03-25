use std::collections::BTreeMap;

use crate::{
    hir::{self, Expr, NameContext},
    mir::{self, MirType, MirTypeInfo},
    sim::{
        observer::GlobalsSchema,
        patch::{PatchId, PatchSchema},
        turtle::{TurtleId, TurtleSchema},
        value::PackedAny,
    },
    util::reflection::Reflect,
    workspace::Workspace,
};

pub struct TypeMapping {
    globals_schema: GlobalsSchema,
    turtle_schema: TurtleSchema,
    patch_schema: PatchSchema,
    workspace_ptr_ty: MirType,
}

impl TypeMapping {
    pub fn new(
        globals_schema: GlobalsSchema,
        turtle_schema: TurtleSchema,
        patch_schema: PatchSchema,
    ) -> Self {
        let workspace_pointee_ty =
            Workspace::mir_type_from_schemas(&globals_schema, &turtle_schema, &patch_schema);
        let workspace_ptr_ty = MirTypeInfo::ptr_to(workspace_pointee_ty);
        Self { globals_schema, turtle_schema, patch_schema, workspace_ptr_ty }
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

    pub fn workspace_ptr_ty(&self) -> MirType {
        self.workspace_ptr_ty.clone()
    }
}

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
    /// breaks from that label should use.
    pub ctrl_flow_constructs: BTreeMap<hir::Label, (mir::Label, mir::LocalId)>,
    /// The local variable that contains the workspace parameter, if it exists.
    pub workspace_param: Option<mir::LocalId>,
    /// The local variable that contains the RNG parameter, if it exists.
    pub rng_param: Option<mir::LocalId>,
    /// The local variable that contains the self parameter, if it exists.
    pub self_param: Option<mir::LocalId>,
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
        let output_ty = expr.output_type(self.hir_names).repr();
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

    pub fn workspace_param(&mut self) -> mir::TypedPlace {
        let local_id = *self.translator.workspace_param.get_or_insert_with(|| {
            let (local_id, _local_decl) = self.mir.create_local(mir::LocalDecl {
                debug_name: Some("workspace".into()),
                ty: self.type_mapping.workspace_ptr_ty.clone(),
            });
            // TODO add as parameter to the function
            local_id
        });
        self.mir.typed_place(local_id)
    }

    // Returns the local variable that contains the self parameter, creating
    // it if it does not exist.
    pub fn self_param(&mut self) -> mir::TypedPlace {
        let local_id = *self.translator.self_param.get_or_insert_with(|| {
            let (local_id, _local_decl) = self.mir.create_local(mir::LocalDecl {
                debug_name: Some("self".into()),
                ty: (PackedAny::TYPE.make_mir_type)(),
            });
            // TODO add as parameter to the function
            local_id
        });
        self.mir.typed_place(local_id)
    }

    /// Returns the local variable that contains self as a turtle id. If
    /// self is not statically known to be a turtle, this will generate runtime
    /// code to downcast to a turtle
    pub fn self_turtle(&mut self) -> mir::TypedPlace {
        let self_param = self.self_param();
        let Some(ty) = self_param.ty.static_ty else {
            panic!("self parameter type is unknown");
        };
        if ty.is::<TurtleId>() {
            self_param
        } else {
            panic!("self parameter is not a turtle");
        }
        // TODO add a branch that attemps to downcast to a turtle
    }

    // Returns the local variable that contains self as a patch id.
    pub fn self_patch(&mut self) -> mir::TypedPlace {
        let self_param = self.self_param();
        let Some(ty) = self_param.ty.static_ty else {
            panic!("self parameter type is unknown");
        };
        if ty.is::<PatchId>() {
            self_param
        } else {
            panic!("self parameter is not a patch");
        }
        // TODO add a branch that attemps to downcast to a patch
    }
}

pub fn hir_to_mir(hir: &hir::Program, type_mapping: &TypeMapping) -> mir::Program {
    let mut builder = mir::ProgramBuilder::new();

    // iterate through each function and convert it to an MIR function
    for (hir_fn_id, hir_fn) in &hir.functions {
        let hir_fn_body = &hir.function_bodies[hir_fn_id];

        // create a builder to track state while translating
        let mut mir_fn_builder = builder.create_function();
        let mut translator = HirToMirFnTranslator::default();

        let mut builder = HirToMirFnBuilder {
            hir_names: NameContext::from_program(hir),
            type_mapping,
            mir: &mut mir_fn_builder,
            translator: &mut translator,
        };

        // add all the nodes to the function body
        let (return_local, _) = builder.mir.create_local(mir::LocalDecl {
            debug_name: Some("return".into()),
            ty: hir_fn.return_ty.repr(),
        });
        builder.with_locals(&hir_fn.parameters, |builder| {
            hir_fn_body.write_mir_execution(builder, return_local);
        });

        mir_fn_builder.set_return(return_local);
        mir_fn_builder.finish();
    }

    todo!()
}
