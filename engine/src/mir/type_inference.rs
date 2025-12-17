use std::{collections::HashMap, mem};

use slotmap::SecondaryMap;
use tracing::trace;

use crate::mir::{
    FunctionId, LocalId, MirVisitor, NlAbstractTy, Node as _, NodeId, NodeKind, Program,
    StatementKind, node::SetLocalVar, visit_mir_function,
};

/// Associates values in the program with all the types that are possible for
/// the value to take on.
#[derive(Default)]
struct ProgramTypes {
    /// Associates each local variable with the types that it can be bound to.
    locals: SecondaryMap<LocalId, NlAbstractTy>,
    /// Associates each function with the types that it can return.
    returns: SecondaryMap<FunctionId, NlAbstractTy>,
    /// Associates each global variable with the types that it can be bound to.
    global_vars: HashMap<usize, NlAbstractTy>,
    /// Associates each patch variable with the types that it can be bound to.
    patch_vars: HashMap<usize, NlAbstractTy>,
    /// Associates each turtle variable with the types that it can be bound to.
    turtle_vars: HashMap<usize, NlAbstractTy>,
}

// TODO(wishlist) this is a very basic algorithm that iteratively runs type
// inference over every single node in the program until convergence. this can
// definitely be made more efficient.
pub fn narrow_types(program: &mut Program) {
    /// A visitor that collects all places where a value is bound and for each
    /// binding, calculates the join of all possible types. This includes
    /// - setting an agent variable or local variable
    /// - passing a parameter to a function
    /// - returning a value from a function
    struct Visitor {
        types: ProgramTypes,
    }
    impl MirVisitor for Visitor {
        fn visit_statement(
            &mut self,
            program: &Program,
            fn_id: FunctionId,
            statement: &StatementKind,
        ) {
            match statement {
                StatementKind::Return { value } => {
                    let ty = program.nodes[*value].output_type(program, fn_id).abstr.unwrap();
                    let existing_ty = self
                        .types
                        .returns
                        .entry(fn_id)
                        .expect("function id should be valid")
                        .or_insert(NlAbstractTy::Bottom);
                    *existing_ty = mem::take(existing_ty).join(ty);
                }
                _ => {} // do nothing
            }
        }

        fn visit_node(&mut self, program: &Program, fn_id: FunctionId, node_id: NodeId) {
            match program.nodes[node_id] {
                NodeKind::SetLocalVar(SetLocalVar { local_id, value }) => {
                    let ty = program.nodes[value].output_type(program, fn_id).abstr.unwrap();
                    let existing_ty = self
                        .types
                        .locals
                        .entry(local_id)
                        .expect("local id should be valid")
                        .or_insert(NlAbstractTy::Bottom);
                    *existing_ty = mem::take(existing_ty).join(ty);
                }
                // TODO(mvp) handle setting agent variables
                _ => {} // do nothing
            }
        }
    }

    let mut visitor = Visitor { types: ProgramTypes::default() };
    // keep running inference until convergence
    let mut changed = true;
    while changed {
        changed = false;

        trace!("running type inference pass");

        // collect type inferences for all nodes in the program
        for fn_id in program.functions.keys() {
            visit_mir_function(&mut visitor, program, fn_id);
        }

        // apply the type inferences to the program and look for changes
        let ProgramTypes { locals, returns, global_vars, patch_vars, turtle_vars } =
            &mut visitor.types;
        for (local_id, new_ty) in locals.drain() {
            let ty = &mut program.locals[local_id].ty;
            assert!(ty.concrete.is_none());
            if ty.abstr.as_ref() != Some(&new_ty) {
                changed = true;
                trace!(
                    "type inference: local variable {:?} type changed from {:?} to {:?}",
                    local_id, ty.abstr, new_ty
                );
                ty.abstr = Some(new_ty);
            }
        }
        for (fn_id, new_ty) in returns.drain() {
            let ty = &mut program.functions[fn_id].return_ty;
            assert!(ty.concrete.is_none());
            if ty.abstr.as_ref() != Some(&new_ty) {
                changed = true;
                trace!(
                    "type inference: function {:?} return type changed from {:?} to {:?}",
                    fn_id, ty.abstr, new_ty
                );
                ty.abstr = Some(new_ty);
            }
        }
        for (global_idx, new_ty) in global_vars.drain() {
            let ty = &mut program.globals[global_idx].ty;
            assert!(ty.concrete.is_none());
            if ty.abstr.as_ref() != Some(&new_ty) {
                changed = true;
                trace!(
                    "type inference: global variable {:?} type changed from {:?} to {:?}",
                    global_idx, ty.abstr, new_ty
                );
                ty.abstr = Some(new_ty);
            }
        }
        for (patch_idx, new_ty) in patch_vars.drain() {
            let ty = &mut program.custom_patch_vars[patch_idx].ty;
            assert!(ty.concrete.is_none());
            if ty.abstr.as_ref() != Some(&new_ty) {
                changed = true;
                trace!(
                    "type inference: patch variable {:?} type changed from {:?} to {:?}",
                    patch_idx, ty.abstr, new_ty
                );
                ty.abstr = Some(new_ty);
            }
        }
        for (turtle_idx, new_ty) in turtle_vars.drain() {
            let ty = &mut program.custom_turtle_vars[turtle_idx].ty;
            assert!(ty.concrete.is_none());
            if ty.abstr.as_ref() != Some(&new_ty) {
                changed = true;
                trace!(
                    "type inference: turtle variable {:?} type changed from {:?} to {:?}",
                    turtle_idx, ty.abstr, new_ty
                );
                ty.abstr = Some(new_ty);
            }
        }
    }
}
