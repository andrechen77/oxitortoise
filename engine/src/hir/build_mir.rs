use std::collections::HashMap;

use crate::{
    exec::CanonExecutionContext,
    hir::{self, Expr},
    mir,
    util::reflection::Reflect as _,
};

pub struct HirToMirFnBuilder<'a, 'b> {
    pub hir: &'a hir::Program,
    pub lir: &'a mut mir::builder::FunctionBuilder<'b>,
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
    pub fn translate_expr(&mut self, expr: &hir::ExprKind) -> mir::LocalId {
        let output_ty = expr.output_type(self.hir).repr();
        // the expression's output will be stored in this local variable
        let output_local =
            self.lir.create_local(mir::LocalDecl { debug_name: None, ty: output_ty });
        expr.write_mir_execution(self, output_local);
        output_local
    }

    // a bunch of boilerplate code to recreate the state of the builder. This is
    // only necessary because we want to keep all builder information packaged
    // together in a single struct. if we passed all the components of
    // HirToLirFnBuilder separately, this would not be necessary.
    pub fn with_inner_statement_seq<T>(
        &mut self,
        f: impl FnOnce(&mut HirToMirFnBuilder<'_, '_>) -> T,
    ) -> (Vec<mir::Statement>, T) {
        self.lir.with_inner_statement_seq(|lir| {
            let mut builder = HirToMirFnBuilder { hir: self.hir, lir, translator: self.translator };
            f(&mut builder)
        })
    }

    /// Returns the local variable that contains the context parameter, creating
    /// it as a function parameter if it does not exist.
    pub fn context_param(&mut self) -> mir::LocalId {
        *self.translator.context_param.get_or_insert_with(|| {
            self.lir.create_local(mir::LocalDecl {
                debug_name: None,
                ty: (&<&mut CanonExecutionContext>::TYPE_INFO).into(),
            })
            // TODO add as parameter to the function
        })
    }
}

pub fn hir_to_mir(hir: &hir::Program) -> mir::Program {
    let mut builder = mir::builder::ProgramBuilder::new();

    // iterate through each function and convert it to an MIR function
    for (hir_fn_id, hir_fn) in &hir.functions {
        // create a builder to track state while translating
        let mut lir_fn_builder = builder.create_function();
        let mut translator = HirToLirFnTranslator::default();

        let mut builder =
            HirToMirFnBuilder { hir, lir: &mut lir_fn_builder, translator: &mut translator };

        // add all the nodes to the function body
        let return_place = builder.translate_expr(&hir_fn.body);

        lir_fn_builder.set_return(return_place);
        lir_fn_builder.finish();
    }

    todo!()
}
