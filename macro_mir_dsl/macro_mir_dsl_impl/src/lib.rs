use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Generics, Token, parse2, punctuated::Punctuated};

use crate::parse::MirIntrinsicSyntax;

mod ast;
mod parse;
mod substitute;

pub fn mir_intrinsic_impl(input: TokenStream) -> syn::Result<TokenStream> {
    let intrinsic: MirIntrinsicSyntax = parse2(input)?;

    let interp_fn =
        substitute::interp::substitute(&intrinsic).unwrap_or_else(syn::Error::into_compile_error);
    let write_mir_fn = substitute::write_mir::substitute(&intrinsic)
        .unwrap_or_else(syn::Error::into_compile_error);
    let intrinsic_ident = intrinsic.signature.ident;

    Ok(quote! {
        mod #intrinsic_ident {
            #interp_fn
            #write_mir_fn
        }
    })
}

/// Returns an identifier intended to only be referenced by other code created
/// by this macro.
fn internal_ident(name: &str) -> syn::Ident {
    syn::Ident::new(&format!("__{}_internal_{}", module_path!(), name), Span::mixed_site())
}

fn ensure_trailing<T>(mut args: Punctuated<T, Token![,]>) -> Punctuated<T, Token![,]> {
    if !args.empty_or_trailing() {
        args.push_punct(Token![,](Span::call_site()));
    }
    args
}

fn combine_generics(a: &Generics, b: &Generics) -> Generics {
    let mut out = Generics::default();

    // keep the lt_token and gt_token from 'a'
    out.lt_token = a.lt_token.clone();
    out.gt_token = a.gt_token.clone();

    // collect lifetime generics
    for lifetime in a.lifetimes() {
        out.params.push(lifetime.clone().into());
    }
    for lifetime in b.lifetimes() {
        out.params.push(lifetime.clone().into());
    }

    // collect type params
    for ty in a.type_params() {
        out.params.push(ty.clone().into());
    }
    for ty in b.type_params() {
        out.params.push(ty.clone().into());
    }

    // add any where clauses from both a and b
    match (&a.where_clause, &b.where_clause) {
        (Some(wc_a), Some(wc_b)) => {
            let mut combined = wc_a.clone();
            for pred in &wc_b.predicates {
                combined.predicates.push(pred.clone());
            }
            out.where_clause = Some(combined);
        }
        (Some(wc), None) | (None, Some(wc)) => {
            out.where_clause = Some(wc.clone());
        }
        (None, None) => {}
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_var() {
        let input = quote! {
            fn project_global_var<T>(var_index: usize)<'a>(context: &'a mut CanonExecutionContext) -> (out: &'a mut T) {
                mir! {
                    out = &mut context
                        .deref::<CanonExecutionContext>()
                        .workspace
                        .deref::<Workspace>()
                        .world
                        .globals
                        .data
                        .dyn_ptr::<RowBuffer>()
                        .dyn_field::<T>(const { type_mapping!().globals_schema().offset_of_field(var_index) });
                }
            }
        };

        let expanded = mir_intrinsic_impl(input);

        match expanded {
            Ok(expanded) => {
                println!("{}", expanded);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    #[test]
    fn arith_op() {
        let input = quote! {
            fn binary_op(op: BinaryOpcode)(lhs: any, rhs: any) -> (out: any) {
                let lhs_ty: Type = type_of!(lhs);
                let rhs_ty: Type = type_of!(rhs);

                let lhs_is_nobody = lhs_ty == Nobody::mir_type();
                let rhs_is_nobody = rhs_ty == Nobody::mir_type();

                // special case comparisons againt nobody
                if (lhs_is_nobody || rhs_is_nobody)
                    && (op == BinaryOpcode::Eq || op == BinaryOpcode::Neq)
                {
                    let negate = match op {
                        BinaryOpcode::Eq => false,
                        BinaryOpcode::Neq => true,
                        _ => unreachable!(),
                    };

                    // short circuit on nobody vs nobody comparison
                    if lhs_is_nobody && rhs_is_nobody {
                        // TODO assign the out type to bool
                        mir! {
                            out = const { if negate { NlBool(false) } else { NlBool(true) } };
                        }
                        return;
                    }

                    // find the operand that is not known to be nobody
                    let (operand, operand_ty) = if lhs_is_nobody {
                        (place_ref!(rhs), rhs_ty)
                    } else {
                        (place_ref!(lhs), lhs_ty)
                    };
                    if operand_ty.is::<OptionPatchId>() {
                        mir! {
                            let negate_rt: bool = const { negate };
                            out = const { OptionPatchId::check_nobody }(negate_rt, &(place_use!(operand).cast::<OptionPatchId>()));
                        }
                    } else {
                        unimplemented!("TODO(mvp) handle nobody check for other operand types: {:?}", rhs_ty);
                    }
                } else if lhs_ty.is::<NlBool>() && rhs_ty.is::<NlBool>() {
                    match op {
                        BinaryOpcode::And => mir! {
                            out = lhs.cast::<NlBool>() && rhs.cast::<NlBool>();
                        },
                        BinaryOpcode::Or => mir! {
                            out = lhs.cast::<NlBool>() || rhs.cast::<NlBool>();
                        },
                        _ => unimplemented!("unsupported operation"),
                    }
                } else {
                    // TODO
                }
            }
        };

        let expanded = mir_intrinsic_impl(input);

        match expanded {
            Ok(expanded) => {
                println!("{}", expanded);
            }
            Err(e) => {
                println!("Error: {}", e.to_compile_error());
            }
        }
    }
}
