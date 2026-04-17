use std::collections::BTreeMap;

use crate::{
    hir::{self, NameContext},
    mir,
};

mod type_mapping;

use tracing::trace;
pub use type_mapping::{TypeMapping, make_type_mapping};

pub struct HirToMirFnBuilder<'a, 'b> {
    pub hir_names: NameContext<'a>,
    // pub hir: &'a hir::Program,
    pub type_mapping: &'a TypeMapping,
    pub mir: &'a mut mir::FunctionBuilder<'b>,
    pub local_translator: &'a mut HirToMirFnTranslator,
    pub user_fn_translator: &'a BTreeMap<hir::FunctionId, mir::FunctionId>,
}

#[derive(Debug, Default)]
pub struct HirToMirFnTranslator {
    pub locals: BTreeMap<hir::LocalId, mir::Place>,
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
                local_translator: self.local_translator,
                user_fn_translator: self.user_fn_translator,
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
            local_translator: self.local_translator,
            user_fn_translator: self.user_fn_translator,
        })
    }

    pub fn with_local_translator<T>(
        &mut self,
        local_translator: &mut HirToMirFnTranslator,
        f: impl FnOnce(&mut HirToMirFnBuilder) -> T,
    ) -> T {
        f(&mut HirToMirFnBuilder {
            hir_names: self.hir_names,
            type_mapping: self.type_mapping,
            mir: self.mir,
            local_translator,
            user_fn_translator: self.user_fn_translator,
        })
    }
}

pub fn hir_to_mir(hir: &hir::Program) -> mir::Program {
    let type_mapping = make_type_mapping(hir);

    let mut builder = mir::ProgramBuilder::new();

    let mut user_fn_translator = BTreeMap::new();
    // for each HIR function, allocate a MIR function id
    for &hir_fn_id in hir.functions.keys() {
        let mir_fn_id = builder.next_function_id();
        user_fn_translator.insert(hir_fn_id, mir_fn_id);

        // create the function stub so that other functions can refer to this
        // function even if it has not been translated yet
        let return_ty = type_mapping.function_return_ty(hir_fn_id);
        builder.insert_function_stub(mir_fn_id, mir::FunctionStub { return_ty });
    }

    // iterate through each function and convert it to an MIR function
    for (hir_fn_id, hir_fn) in &hir.functions {
        trace!("translating function {:?}", hir_fn_id);

        let mir_fn_id = user_fn_translator[hir_fn_id];
        let hir_fn_body = &hir.function_bodies[hir_fn_id];

        let name_context = NameContext::from_program(hir);
        let mut mir_fn_builder = builder.create_function(mir_fn_id);
        let mut local_translator = HirToMirFnTranslator::default();
        let hir::Function { debug_name, parameters, return_ty, is_entrypoint } = hir_fn;

        translate_function(
            name_context,
            &type_mapping,
            &mut mir_fn_builder,
            &mut local_translator,
            &user_fn_translator,
            &parameters,
            hir_fn_body,
        );

        mir_fn_builder.finish();
    }

    builder.finish()
}

pub fn translate_function(
    hir_names: NameContext,
    type_mapping: &TypeMapping,
    mir_fn_builder: &mut mir::FunctionBuilder,
    local_translator: &mut HirToMirFnTranslator,
    user_fn_translator: &BTreeMap<hir::FunctionId, mir::FunctionId>,
    parameters: &BTreeMap<hir::LocalId, hir::LocalDecl>,
    body: &hir::ExprKind,
) {
    let mut builder = HirToMirFnBuilder {
        hir_names,
        type_mapping: &type_mapping,
        mir: mir_fn_builder,
        local_translator,
        user_fn_translator,
    };

    // add all the function parameters to the function builder
    for (hir_param_id, param_decl) in parameters {
        let ty = builder.type_mapping.local_var_ty(*hir_param_id);
        let mir_param_id = builder.mir.create_parameter(mir::LocalDecl {
            debug_name: Some(param_decl.debug_name.clone()),
            ty,
        });
        builder.local_translator.locals.insert(*hir_param_id, mir_param_id.place());
    }

    // add all the nodes to the function body
    let return_value = builder.with_locals(parameters, |builder| translate_expr(builder, body));

    if let Some(return_value) = return_value {
        mir_fn_builder.set_return(return_value);
    }
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
    use hir::ExprKind as E;
    match expr {
        // control flow
        E::Block(block) => block.write_mir_execution(builder),
        E::Break(break_expr) => break_expr.write_mir_execution(builder),
        E::IfElse(if_else) => if_else.write_mir_execution(builder),

        // local variables
        E::Scope(scope) => scope.write_mir_execution(builder),
        E::GetLocalVar(get_local_var) => get_local_var.write_mir_execution(builder),
        E::SetLocalVar(set_local_var) => set_local_var.write_mir_execution(builder),

        // agent variables
        E::GetGlobalVar(get_global_var) => get_global_var.write_mir_execution(builder),
        E::GetTurtleVar(get_turtle_var) => get_turtle_var.write_mir_execution(builder),
        E::SetTurtleVar(set_turtle_var) => set_turtle_var.write_mir_execution(builder),
        E::GetPatchVar(get_patch_var) => get_patch_var.write_mir_execution(builder),
        E::SetPatchVar(set_patch_var) => set_patch_var.write_mir_execution(builder),

        // arith ops
        E::BinaryArith(binary_arith) => binary_arith.write_mir_execution(builder),
        E::BinaryCmp(binary_cmp) => binary_cmp.write_mir_execution(builder),
        E::BinaryBool(binary_bool) => binary_bool.write_mir_execution(builder),
        E::LogicalNot(logical_not) => logical_not.write_mir_execution(builder),
        E::Negate(negate) => negate.write_mir_execution(builder),

        // other
        E::Ask(ask) => ask.write_mir_execution(builder),
        E::Of(of) => of.write_mir_execution(builder),
        E::CreateTurtles(create_turtles) => create_turtles.write_mir_execution(builder),
        E::ClearAll(clear_all) => clear_all.write_mir_execution(builder),
        E::Distancexy(distancexy) => distancexy.write_mir_execution(builder),
        E::AdvanceTick(advance_tick) => advance_tick.write_mir_execution(builder),
        E::Diffuse(diffuse) => diffuse.write_mir_execution(builder),
        E::ScaleColor(scale_color) => scale_color.write_mir_execution(builder),
        E::TurtleForward(turtle_forward) => turtle_forward.write_mir_execution(builder),
        E::TurtleRotate(turtle_rotate) => turtle_rotate.write_mir_execution(builder),
        E::PatchAt(patch_at) => patch_at.write_mir_execution(builder),
        E::GetTick(get_tick) => get_tick.write_mir_execution(builder),
        E::ResetTicks(reset_ticks) => reset_ticks.write_mir_execution(builder),
        E::MaxPxcor(max_pxcor) => max_pxcor.write_mir_execution(builder),
        E::MaxPycor(max_pycor) => max_pycor.write_mir_execution(builder),
        E::OneOf(one_of) => one_of.write_mir_execution(builder),
        E::ListLiteral(list_literal) => list_literal.write_mir_execution(builder),
        E::NumberLiteral(number_literal) => number_literal.write_mir_execution(builder),
        E::UnitLiteral(unit_literal) => Some(unit_literal.write_mir_execution(builder)),
        E::CallUserFn(call_user_fn) => call_user_fn.write_mir_execution(builder),
        E::SetDefaultShape(set_default_shape) => set_default_shape.write_mir_execution(builder),
        E::RandomInt(random_int) => random_int.write_mir_execution(builder),
        E::CanMove(can_move) => can_move.write_mir_execution(builder),
        E::PatchRelative(patch_relative) => patch_relative.write_mir_execution(builder),

        // TODO fill all other match arms
        E::Agentset(_agentset) => todo!(),
        E::StringLiteral(_string_literal) => todo!(),
        E::NobodyLiteral(_nobody_literal) => todo!(),
        E::OffsetDistanceByHeading(_offset_distance_by_heading) => todo!(),
        E::Nop(_nop) => None,

        E::Closure(_closure) => todo!("TODO implement standalone closure"),
        E::EuclideanDistanceNoWrap(_euclidean_distance_no_wrap) => unimplemented!(),
        E::PointConstructor(_point_constructor) => unimplemented!(),
    }
}
