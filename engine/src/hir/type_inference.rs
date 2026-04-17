use std::{collections::BTreeMap, fmt::Debug, iter};

use tracing::{trace, warn};

use crate::{
    hir::{
        self, Expr, ExprKind, Function, FunctionId, LocalId, NameContext, NlAbstractTy,
        NlAbstractTyAtom, Program, expr,
    },
    sim::{patch::PatchVarDesc, turtle::TurtleVarDesc},
};

// TODO(mvp) handle cases where an agent variable is not set, but keeps its
// starting default value of 0. in that case, we should join the type with
// Float. however, if the variable is immediately set when the agent is created,
// then it should be as if it never got the default value at all.

// TODO(wishlist) this is a very basic algorithm that iteratively runs type
// inference over every single node in the program until convergence. this can
// definitely be made more efficient.
pub fn narrow_types(program: &mut Program) {
    let mut changed = true;
    while changed {
        changed = narrow_types_once(program);
    }
}

/// For a given static point in the program, associates all values in scopes
/// with the types that are possible for the value to take on.
#[derive(Default)]
struct ProgramTypes {
    /// Associates each global variable with the types that it can be bound to.
    global_vars: BTreeMap<usize, NlAbstractTy>,
    /// Associates each patch variable with the types that it can be bound to.
    patch_vars: BTreeMap<usize, NlAbstractTy>,
    /// Associates each turtle variable with the types that it can be bound to.
    turtle_vars: BTreeMap<usize, NlAbstractTy>,
    /// Associates each local variable with the types that it can be bound to.
    /// Inside a function body, this is also used to track assignments to
    /// function parameters, but it must be manually joined with the `fn_params`
    /// field outside a function body to get the full picture of all the types
    /// that the function parameters can take.
    locals: BTreeMap<LocalId, NlAbstractTy>,
    /// Associates each function parameter with the types that it can be bound to.
    fn_params: BTreeMap<(FunctionId, LocalId), NlAbstractTy>,
    /// Tracks modifications to the type that are not reflected in the other
    /// fields. For example, if a local variable goes out of scope and is
    /// removed from the `locals` field, but its type had changed, this flag
    /// should be set to notify of a change.
    changed: bool,
}

impl ProgramTypes {
    /// Executes the given function with the given local variables added to the
    /// scope. Although this takes a mutable reference to the table of locals,
    /// this does not actually mutate the locals, it just passes the mutable
    /// reference to the inner function.
    fn with_locals(
        &mut self,
        locals: impl Iterator<Item = hir::LocalId>,
        f: impl FnOnce(&mut Self),
    ) {
        // any new variables that have the same LocalId as an existing variable
        // will shadow the existing variable. replace them in the map, but keep
        // track of them so that we can restore them after the inner function
        // call
        let mut old_pairs = Vec::new();
        for local_id in locals {
            // initialize the local variables to the "bottom" type so that
            // successive joins give the minimal type compatible with all
            // assignments
            let old_ty = self.locals.insert(local_id, NlAbstractTy::bottom());
            old_pairs.push((local_id, old_ty));
        }

        f(self);

        // restore the values to what they were before the inner function call
        for (local_id, ty) in old_pairs {
            if let Some(ty) = ty {
                self.locals.insert(local_id, ty);
            } else {
                self.locals.remove(&local_id);
            }
        }
    }
}

fn narrow_types_once(program: &mut Program) -> bool {
    trace!("performing single type inference pass");

    let mut changed = false;
    let mut types = ProgramTypes::default();

    let fn_ids = program.functions.keys().copied().collect::<Vec<_>>();

    for fn_id in &fn_ids {
        trace!("narrowing types for function {:?}", fn_id);

        // split the program into an immutable "names context" and a mutable
        // section of function bodies
        let (names, function_bodies) = NameContext::from_program_mut(program);
        let Function { parameters: fn_params, is_entrypoint, .. } = &names.functions()[fn_id];
        let body_expr = function_bodies.get_mut(fn_id).unwrap();
        // visit the function body and narrow types for inner expressions
        types.with_locals(fn_params.keys().copied().collect::<Vec<_>>().into_iter(), |types| {
            // to prevent entrypoint functions from having their
            // parameter types narrowed, we reassert their parameter
            // types to emulate the fact that they can be called with
            // parameters of any value (subject to the initial type
            // constraint with which they were declared)
            if *is_entrypoint {
                for local_id in fn_params.keys() {
                    let og_declared_ty = fn_params[local_id].ty.clone();
                    join_type(&mut types.fn_params, (*fn_id, *local_id), og_declared_ty);
                }
            }

            narrow_types_expr(types, names.with_locals(fn_params), body_expr);

            // the call to narrow_types_expr will update the types of any
            // bindings local to the expression, but this does not include
            // function parameters (which are free variables from the
            // perspective of the function body expression). however it will
            // still collect all local assignments to function parameter
            // variables, if any exist, so we can join these into the
            // `types.fn_params` field to persist them beyond the function body.
            for local_id in fn_params.keys() {
                // the minimal type covering all assignments to this variable
                // inside the function body.
                let local_assigns =
                    types.locals.remove(local_id).unwrap_or_else(NlAbstractTy::bottom);
                join_type(&mut types.fn_params, (*fn_id, *local_id), local_assigns);
            }
        });

        // check if the return type of the function is different and update it
        // if it is narrowable
        let new_return_ty = body_expr.output_type(names);
        changed |= narrow_types_specific(
            &mut program.functions,
            |program_functions, fn_id| &mut program_functions.get_mut(fn_id).unwrap().return_ty,
            iter::once((fn_id, new_return_ty)),
        );
    }

    let ProgramTypes {
        global_vars,
        patch_vars,
        turtle_vars,
        locals,
        fn_params,
        changed: invisible_changes,
    } = types;

    // mark invisible changes to local variables
    changed |= invisible_changes;

    assert!(
        locals.is_empty(),
        "all local variables should have gone out of scope since we are not traversing a function body"
    );

    // narrow global variables
    changed |= narrow_types_specific(
        &mut program.global_vars,
        |global_vars, global_idx| &mut global_vars[global_idx].ty,
        global_vars.into_iter(),
    );

    // narrow patch variables
    changed |= narrow_types_specific(
        &mut program.custom_patch_vars,
        |custom_patch_vars, patch_idx| &mut custom_patch_vars[patch_idx].ty,
        patch_vars.into_iter(),
    );

    // narrow turtle variables
    changed |= narrow_types_specific(
        &mut program.custom_turtle_vars,
        |custom_turtle_vars, turtle_idx| &mut custom_turtle_vars[turtle_idx].ty,
        turtle_vars.into_iter(),
    );

    // narrow function parameters
    changed |= narrow_types_specific(
        &mut program.functions,
        |program_functions, (fn_id, local_id)| {
            &mut program_functions
                .get_mut(&fn_id)
                .unwrap()
                .parameters
                .get_mut(&local_id)
                .unwrap()
                .ty
        },
        fn_params.into_iter(),
    );

    // return whether any changes were made
    changed
}

/// Visit every expression in the function body, looking for every time a
/// variable is set. Any local variables scope-limited to the given expression
/// will be set to the minimal type compatible with all assignments to that
/// variable.
fn narrow_types_expr(types: &mut ProgramTypes, names: NameContext, expr: &mut ExprKind) {
    match expr {
        ExprKind::Scope(expr::Scope { locals, inner }) => {
            let local_ids = locals.keys().copied().collect::<Vec<_>>();
            types.with_locals(local_ids.iter().copied(), |types| {
                // collect all assignments to these locals
                narrow_types_expr(types, names.with_locals(locals), inner);

                // we have seen all assignments to these local variables, so
                // now we can take this information out of the map and update
                // the actual types of the local variables
                narrow_types_specific(
                    locals,
                    |locals, local_id| &mut locals.get_mut(&local_id).unwrap().ty,
                    local_ids
                        .iter()
                        .map(|&local_id| (local_id, types.locals.remove(&local_id).unwrap())),
                );
            });
        }
        ExprKind::Ask(expr::Ask { workspace, rng, recipients, body })
        | ExprKind::Of(expr::Of { workspace, rng, recipients, body }) => 'a: {
            narrow_types_expr(types, names, workspace);
            narrow_types_expr(types, names, rng);
            narrow_types_expr(types, names, recipients);

            let recipients_ty = recipients.output_type(names);
            let closure_self_ty = if let Some(NlAbstractTyAtom::Agentset { agent_type }) =
                recipients_ty.get_union().unwrap().get_atom()
            {
                agent_type.as_ref().clone()
            } else {
                recipients_ty
            };

            // find which parameter corresponds to the self type
            let ExprKind::Closure(closure) = body.as_mut() else {
                warn!("expected ask/of body to be a closure literal, got: {:?}", body);
                break 'a;
            };
            let self_param_id = *closure.parameters.keys().nth(2).unwrap(); // parameters[2] is the self type

            narrow_types_in_closure(
                types,
                names,
                closure,
                iter::once((self_param_id, closure_self_ty)),
            );
        }
        ExprKind::Closure(closure) => {
            narrow_types_in_closure(types, names, closure, iter::empty());
        }
        ExprKind::SetLocalVar(expr::SetLocalVar { local_id, value }) => {
            narrow_types_expr(types, names, value);

            let ty = value.output_type(names);
            trace!("found assignment to local variable {:?}: {:?}", local_id, ty);
            join_type(&mut types.locals, *local_id, ty);
        }
        // TODO(mvp) handle setting global variables and link variables
        ExprKind::SetTurtleVar(expr::SetTurtleVar {
            var: TurtleVarDesc::Custom(var_idx),
            workspace,
            turtle,
            value,
        }) => {
            narrow_types_expr(types, names, workspace);
            narrow_types_expr(types, names, turtle);
            narrow_types_expr(types, names, value);

            let ty = value.output_type(names);
            trace!("found assignment to turtle variable {:?}: {:?}", var_idx, ty);
            join_type(&mut types.turtle_vars, *var_idx, ty);
        }
        ExprKind::SetPatchVar(expr::SetPatchVar {
            var: PatchVarDesc::Custom(var_idx),
            workspace,
            patch,
            value,
        }) => {
            narrow_types_expr(types, names, workspace);
            narrow_types_expr(types, names, patch);
            narrow_types_expr(types, names, value);

            let ty = value.output_type(names);
            trace!("found assignment to patch variable {:?}: {:?}", var_idx, ty);
            join_type(&mut types.patch_vars, *var_idx, ty);
        }
        ExprKind::CallUserFn(expr::CallUserFn { target, args }) => {
            for arg in args.iter_mut() {
                narrow_types_expr(types, names, arg);
            }

            let target_params = &names.functions()[target].parameters;
            assert_eq!(
                args.len(),
                target_params.len(),
                "number of arguments to call user fn must match number of parameters"
            );
            for (arg, target_param) in args.iter().zip(target_params.keys()) {
                let ty = arg.output_type(names);
                trace!(
                    "found assignment to function parameter {:?}: {:?}",
                    (*target, *target_param),
                    ty
                );
                join_type(&mut types.fn_params, (*target, *target_param), ty);
            }
        }
        other => other.visit_children_mut(|child| narrow_types_expr(types, names, child)),
    }
}

/// Narrows the types of the parameters and body of the closure, recursively
/// visiting subexpressions.
///
/// `arg_restrictions` maps a local ID corresponding to a closure parameter to
/// the type of values that can be passed to that parameter. This is used to
/// narrow the parameter types when additional context is known about how the
/// closure is being used. If a closure parameter is not mapped, that implicitly
/// means there is no restriction on the type of values that can be passed in
/// that parameter, so that parameter type should not be narrowed.
fn narrow_types_in_closure(
    types: &mut ProgramTypes,
    names: NameContext,
    expr: &mut expr::Closure,
    arg_restrictions: impl Iterator<Item = (LocalId, NlAbstractTy)>,
) {
    let expr::Closure { parameters, body, captures: _ } = expr;

    let parameter_local_ids = parameters.keys().copied().collect::<Vec<_>>();
    types.with_locals(parameter_local_ids.iter().copied(), |types| {
        // collect all assignments inside the closure body
        narrow_types_expr(types, names.with_locals(parameters), body);

        // unlike with the Scope expression, we don't replace the types of *all*
        // parameters here because we haven't actually seen all the assigments
        // to the parameters; specifically we don't know all the arguments that
        // the closure could be called with. thus, we only narrow the closure
        // parameter types for which we have been given external information
        // about all the ways that the closure could be called.
        narrow_types_specific(
            parameters,
            |parameters, local_id| &mut parameters.get_mut(&local_id).unwrap().ty,
            arg_restrictions.map(|(local_id, mut external_restriction)| {
                external_restriction.join(types.locals.remove(&local_id).unwrap());
                (local_id, external_restriction)
            }),
        );
    });
}

/// For each local variable declaration in `actual_types`, narrow the actual
/// type of the local variable to the type from new_types. This will not widen
/// the actual type (which is considered to be an upper bound) even if the type
/// from new_types is a more general type.
fn narrow_types_specific<K: Debug + Copy, T>(
    // these two arguments are used to function as an `IndexMut<K, Output =
    // NlAbstractTy>` (which requires disgusting newtype wrappers), or a
    // `FnMut(K) -> &mut NlAbstractTy` (which doesn't properly bind the lifetime
    // of the returned reference to the data being borrowed). I would love to
    // know if there is a way to make either of these work
    actual_types_storage: &mut T,
    get_actual_ty: impl Fn(&mut T, K) -> &mut NlAbstractTy,
    new_types: impl Iterator<Item = (K, NlAbstractTy)>,
) -> bool {
    let mut changed = false;
    for (id, new_ty) in new_types {
        let actual_ty = get_actual_ty(actual_types_storage, id);

        if actual_ty.meet(new_ty) {
            changed = true;
            trace!("type inference: variable {:?} type narrowed to {}", id, actual_ty);
        }
    }
    changed
}

/// Given a mapping tracking possible types that a variable can take, expands
/// the possibilities to include the incoming type.
fn join_type<K: Copy + Ord>(
    ty_map: &mut BTreeMap<K, NlAbstractTy>,
    var_id: K,
    incoming_ty: NlAbstractTy,
) {
    let existing_ty = ty_map.entry(var_id).or_insert_with(NlAbstractTy::bottom);
    existing_ty.join(incoming_ty);
}
