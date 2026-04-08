use std::collections::BTreeMap;

use crate::{
    hir::{self, NameContext},
    mir,
    util::reflection::CloneKind,
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
        let return_value =
            builder.with_locals(&hir_fn.parameters, |builder| translate_expr(builder, hir_fn_body));

        if let Some(return_value) = return_value {
            mir_fn_builder.set_return(return_value);
        }
        mir_fn_builder.finish();
    }

    todo!()
}

/// Writes the MIR statements that correspond to the evaluation of the given
/// expression. The return value of this function is the local variable where
/// the return value of the expression can be accessed. If the expression never
/// returns, this should be `None`. If the expression returns unit, this should
/// be a unit-typed local variable.
pub fn translate_expr(
    builder: &mut HirToMirFnBuilder,
    expr: &hir::ExprKind,
) -> Option<mir::LocalId> {
    match expr {
        // control flow
        hir::ExprKind::Block(block) => block.write_mir_execution(builder),
        hir::ExprKind::Break(break_expr) => break_expr.write_mir_execution(builder),
        hir::ExprKind::IfElse(if_else) => if_else.write_mir_execution(builder),

        // agent variables
        hir::ExprKind::GetGlobalVar(get_global_var) => get_global_var.write_mir_execution(builder),
        hir::ExprKind::GetTurtleVar(get_turtle_var) => get_turtle_var.write_mir_execution(builder),
        hir::ExprKind::SetTurtleVar(set_turtle_var) => set_turtle_var.write_mir_execution(builder),
        hir::ExprKind::GetPatchVar(get_patch_var) => get_patch_var.write_mir_execution(builder),
        hir::ExprKind::SetPatchVar(set_patch_var) => set_patch_var.write_mir_execution(builder),

        // arith ops
        hir::ExprKind::BinaryArith(binary_arith) => binary_arith.write_mir_execution(builder),
        hir::ExprKind::BinaryCmp(binary_cmp) => binary_cmp.write_mir_execution(builder),
        hir::ExprKind::BinaryBool(binary_bool) => binary_bool.write_mir_execution(builder),
        hir::ExprKind::LogicalNot(logical_not) => logical_not.write_mir_execution(builder),
        hir::ExprKind::Negate(negate) => negate.write_mir_execution(builder),

        hir::ExprKind::Ask(ask) => ask.write_mir_execution(builder),
        hir::ExprKind::CreateTurtles(create_turtles) => create_turtles.write_mir_execution(builder),

        hir::ExprKind::ClearAll(clear_all) => clear_all.write_mir_execution(builder),

        _ => todo!("TODO(mvp) write MIR execution for {:?}", expr),
    }
}

/// Moves a value from one place to another. The source place is not
/// deinitialized. The destination place will not be deinitialized (i.e. it is
/// assumed to be uninitialized). Useful for loading variables from memory.
pub fn clone_to_new(
    builder: &mut mir::FunctionBuilder,
    src: mir::Place,
    clone_kind: &CloneKind,
) -> mir::LocalId {
    let dst =
        builder.create_local(mir::LocalDecl { debug_name: None, ty: builder.type_of_place(&src) });
    match clone_kind {
        CloneKind::Copy => builder.add_operation_with_dst(
            dst.place(),
            mir::Operation::Operand(mir::PlaceOperand::Copy(src)),
        ),
        CloneKind::Dynamic { clone_fn_info, .. } => builder.add_operation_with_dst(
            dst.place(),
            mir::Operation::CallHostFunction {
                function: clone_fn_info,
                args: vec![mir::PlaceOperand::Borrow(src)],
            },
        ),
        CloneKind::None => {
            panic!("Cannot load a variable from memory that is neither Copy nor Clone");
        }
    }
    dst
}

/// Moves a value from one place to another. This may potentially destroy the
/// source place if it is not Copy. The destination place is considered
/// initialized and will be deinitialized before the value is moved in.
pub fn move_to_init(builder: &mut mir::FunctionBuilder, dst_init: mir::Place, src: mir::LocalId) {
    // deinitialize the destination place
    builder.add_statement(mir::Statement::Elementary(mir::ElementaryStatement::Drop {
        src: dst_init.clone(),
    }));

    // move the value into the place
    builder.add_operation_with_dst(dst_init, mir::Operation::Operand(mir::PlaceOperand::Move(src)));
}
