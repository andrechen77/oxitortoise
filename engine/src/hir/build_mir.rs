use std::collections::HashMap;

use crate::{
    hir::{self, Expr},
    mir,
};

pub struct HirToLirFnBuilder<'a, 'b> {
    pub hir: &'a hir::Program,
    pub lir: &'a mut mir::builder::FunctionBuilder<'b>,
    pub translator: &'a mut HirToLirFnTranslator,
}

pub struct HirToLirFnTranslator {
    pub locals: HashMap<hir::LocalId, mir::LocalId>,
    /// Maps each HIR label to an MIR label, as well as the output local that
    /// breaks from that label should use.
    pub ctrl_flow_constructs: HashMap<hir::Label, (mir::Label, mir::LocalId)>,
}

impl<'a, 'b> HirToLirFnBuilder<'a, 'b> {
    /// Generates MIR statements to evaluate the given HIR expression with all
    /// its dependencies. The return value is stored in the provided out local
    /// or a new local variable. Any temporary values created during evaluation
    /// are stored in new local variables; this function does not reuse existing
    /// locals for temporaries.
    ///
    /// There is currently no checking if dependencies have already been
    /// evaluated.
    pub fn translate_expr(
        &mut self,
        expr: &hir::ExprKind,
        local_out: Option<mir::LocalId>,
    ) -> mir::LocalId {
        // the expression's output will be stored in this local variable
        let output_local = local_out.unwrap_or_else(|| {
            let output_ty = expr.output_type(self.hir).repr();
            self.lir.create_local(mir::LocalDecl { debug_name: None, ty: output_ty })
        });
        expr.write_mir_execution(self, output_local);
        output_local
    }

    // a bunch of boilerplate code to recreate the state of the builder. This is
    // only necessary because we want to keep all builder information packaged
    // together in a single struct. if we passed all the components of
    // HirToLirFnBuilder separately, this would not be necessary.
    pub fn with_inner_statement_seq<T>(
        &mut self,
        f: impl FnOnce(&mut HirToLirFnBuilder<'_, '_>) -> T,
    ) -> (Vec<mir::Statement>, T) {
        self.lir.with_inner_statement_seq(|lir| {
            let mut builder = HirToLirFnBuilder { hir: self.hir, lir, translator: self.translator };
            f(&mut builder)
        })
    }
}

pub fn hir_to_mir(hir: &hir::Program) -> mir::Program {
    let mut builder = mir::builder::ProgramBuilder::new();

    // iterate through each function and convert it to an MIR function
    for (hir_fn_id, hir_fn) in &hir.functions {
        // create a builder to track state while translating
        let mut lir_fn_builder = builder.create_function();
        let mut translator =
            HirToLirFnTranslator { locals: HashMap::new(), ctrl_flow_constructs: HashMap::new() };

        let mut builder =
            HirToLirFnBuilder { hir, lir: &mut lir_fn_builder, translator: &mut translator };

        // add all the nodes to the function body
        let return_place = builder.translate_expr(&hir_fn.body, None);

        lir_fn_builder.set_return(return_place);
        lir_fn_builder.finish();
    }

    todo!()
}
