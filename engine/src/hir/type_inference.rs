use std::{collections::BTreeMap, fmt::Display, mem};

use tracing::trace;

use crate::hir::{Expr, ExprKind, FunctionId, LocalId, NameContext, NlAbstractTy, Program, expr};

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
    fn with_locals(&mut self, locals: impl Iterator<Item = LocalId>, f: impl FnOnce(&mut Self)) {
        // any new variables that have the same LocalId as an existing variable
        // will shadow the existing variable. replace them in the map, but keep
        // track of them so that we can restore them after the inner function
        // call
        let mut old_pairs = Vec::new();
        for local_id in locals {
            // initialize the local variables to the "bottom" type so that
            // successive joins give the minimal type compatible with all
            // assignments
            let old_ty = self.locals.insert(local_id, NlAbstractTy::Bottom);
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
    let mut changed = false;
    let mut types = ProgramTypes::default();

    let fn_ids = program.functions.keys().copied().collect::<Vec<_>>();

    for fn_id in &fn_ids {
        // split the program into an immutable "names context" and a mutable
        // section of function bodies
        let (names, function_bodies) = NameContext::from_program_mut(program);
        let fn_params = &names.functions()[fn_id].parameters;
        let body_expr = function_bodies.get_mut(fn_id).unwrap();
        // visit the function body and narrow types for inner expressions
        types.with_locals(fn_params.keys().copied().collect::<Vec<_>>().into_iter(), |types| {
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
                // inside the function body
                let local_assigment_type =
                    types.locals.remove(local_id).unwrap_or(NlAbstractTy::Bottom);
                join_type(&mut types.fn_params, (*fn_id, *local_id), local_assigment_type);
            }
        });

        // check if the return type of the function is different and update it
        // if it is narrowable
        let new_return_ty = body_expr.output_type(names);
        let old_return_ty = &mut program.functions.get_mut(fn_id).unwrap().return_ty;
        if new_return_ty != *old_return_ty {
            changed = true;
            trace!(
                "type inference: function {:?} return type changed from {:?} to {:?}",
                fn_id, old_return_ty, new_return_ty
            );
            *old_return_ty = new_return_ty;
        }
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
    if invisible_changes {
        changed = true;
    }

    // narrow global variables
    replace_types(
        &mut program.global_vars,
        |global_vars, global_idx| &mut global_vars[global_idx].ty,
        global_vars.into_iter(),
    );

    // narrow patch variables
    replace_types(
        &mut program.custom_patch_vars,
        |custom_patch_vars, patch_idx| &mut custom_patch_vars[patch_idx].ty,
        patch_vars.into_iter(),
    );

    // narrow turtle variables
    replace_types(
        &mut program.custom_turtle_vars,
        |custom_turtle_vars, turtle_idx| &mut custom_turtle_vars[turtle_idx].ty,
        turtle_vars.into_iter(),
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
                replace_types(
                    locals,
                    |locals, local_id| &mut locals.get_mut(&local_id).unwrap().ty,
                    local_ids
                        .iter()
                        .map(|&local_id| (local_id, types.locals.remove(&local_id).unwrap())),
                );
            });
            // early return so that we don't visit the inner expression twice
            return;
        }
        ExprKind::SetLocalVar(expr::SetLocalVar { local_id, value }) => {
            let ty = value.output_type(names);
            join_type(&mut types.locals, *local_id, ty);
        }
        // TODO handle setting agent variables
        _ => {} // do nothing
    }
    expr.visit_children_mut(|child| narrow_types_expr(types, names, child));
}

/// For each local variable declaration in `actual_types`, update the actual
/// type of the local variable with the type from new_types
fn replace_types<K: Display + Copy, T>(
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
        if *actual_ty != new_ty {
            changed = true;
            trace!("type inference: variable {} type changed from {} to {}", id, actual_ty, new_ty);
            *actual_ty = new_ty;
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
    let existing_ty = ty_map.entry(var_id).or_insert(NlAbstractTy::Bottom);
    *existing_ty = mem::take(existing_ty).join(incoming_ty);
}
