use std::mem;

use crate::hir::{Expr, ExprKind, LocalId, expr};

fn inline_of(expr_ref: &mut ExprKind) {
    let expr = mem::take(expr_ref);

    let ExprKind::Of(of) = expr else {
        *expr_ref = expr;
        expr_ref.visit_children_mut(|child| inline_of(child));
        return;
    };

    let expr::Of { workspace, rng, recipients, body } = of;

    let ExprKind::Closure(expr::Closure { parameters, body, captures: _ }) = *body else {
        panic!("expected of body to be a closure literal, got: {:?}", body);
    };

    assert!(parameters.len() == 3, "of body must have 3 parameters, got: {:?}", parameters);
    let mut parameters_iter = parameters.keys();
    let workspace_param = *parameters_iter.next().unwrap();
    let rng_param = *parameters_iter.next().unwrap();
    let self_param = *parameters_iter.next().unwrap();

    todo!()
}

fn remap_local_ids(expr: &mut ExprKind, mut mapping: impl FnMut(LocalId) -> Option<LocalId>) {
    match expr {
        ExprKind::SetLocalVar(expr::SetLocalVar { local_id, value }) => {
            if let Some(new_local_id) = mapping(*local_id) {
                *local_id = new_local_id;
            }
        }
        ExprKind::GetLocalVar(expr::GetLocalVar { local_id }) => {
            if let Some(new_local_id) = mapping(*local_id) {
                *local_id = new_local_id;
            }
        }
        other => other.visit_children_mut(|child| remap_local_ids(child, &mut mapping)),
    }
}
