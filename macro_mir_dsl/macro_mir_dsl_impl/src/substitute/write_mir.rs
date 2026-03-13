use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;

use crate::{
    ast::{
        Assign, Block, Break, Drop, IfElse, LocalDecl, MirBody, Operation, Place, PlaceOperand,
        Stmt, parse_mir_body, parse_place_ref, parse_type_of,
    },
    combine_generics, ensure_trailing, internal_ident,
    parse::{MirIntrinsicSignatureSyntax, MirIntrinsicSyntax},
    substitute::substitute_internal,
};

fn builder_ident() -> syn::Ident {
    internal_ident("builder")
}

pub fn substitute(intrinsic: &MirIntrinsicSyntax) -> syn::Result<TokenStream> {
    let MirIntrinsicSignatureSyntax {
        attrs,
        vis,
        compile_time_generics,
        compile_time_args,
        runtime_generics,
        runtime_args,
        return_value_decl,
        ..
    } = &intrinsic.signature;
    let generics = combine_generics(compile_time_generics, runtime_generics);
    let compile_time_args = ensure_trailing(compile_time_args.clone());
    let mut runtime_args = runtime_args.clone();
    for arg in runtime_args.iter_mut() {
        let syn::FnArg::Typed(pat_type) = arg else {
            panic!("expected typed argument");
        };
        pat_type.ty = Box::new(syn::parse_quote! { crate::mir::Place });
    }
    let runtime_args = ensure_trailing(runtime_args);
    let return_pat = return_value_decl.pat.clone();

    let body =
        substitute_internal(&intrinsic.content, sub_mir_block, sub_type_of, sub_place_ref, None)?;

    // build the write_mir function
    let builder_ident = builder_ident();
    let full_fn = quote! {
        #(#attrs)*
        #[allow(unused_braces)]
        #vis
        fn write_mir
        #generics(
            #builder_ident: &mut crate::mir::builder::FunctionBuilder<'_>,
            #compile_time_args
            #runtime_args
            #return_pat: crate::mir::Place
        )
        #body
    };
    Ok(full_fn)
}

fn sub_type_of(m: &syn::ExprMacro) -> syn::Result<TokenStream> {
    let inner_expr = parse_type_of(m)?;
    let builder_ident = builder_ident();
    Ok(quote! { #builder_ident.type_of_place(&#inner_expr) })
}

fn sub_place_ref(m: &syn::ExprMacro) -> syn::Result<TokenStream> {
    let inner_expr = parse_place_ref(m)?;
    // a "place ref" in the write_mir implementation is the mir::Place object
    // itself
    Ok(quote! { #inner_expr })
}

/// Given the token stream input to a `mir!` macro, returns the code to replace
/// in the write_mir implementation. The first element of the tuple is code
/// that should be hoisted into the top of the function where the mir block
/// appears (typically local declarations). The second element of the tuple is
/// the code to replace the `mir!` macro with.
fn sub_mir_block(m: &syn::Macro) -> syn::Result<(TokenStream, TokenStream)> {
    let MirBody { local_decls, statements } = parse_mir_body(m)?;

    let builder_ident = builder_ident();
    let local_decls = local_decls
        .into_iter()
        .map(|LocalDecl { let_token: _, name, colon_token: _, ty }| {
            quote! {
                let #name: crate::mir::Place = #builder_ident.create_local(crate::mir::LocalDecl {
                    debug_name: Some(stringify!(#name).into()),
                    ty: <#ty as crate::mir::reflection::MirReflect>::mir_type(),
                }).into();
            }
        })
        .collect();

    let statements = statements
        .into_iter()
        .map(|statement| sub_stmt(statement).unwrap_or_else(syn::Error::into_compile_error));

    Ok((local_decls, quote! { { #(#statements)* } }))
}

fn sub_stmt(stmt: Stmt) -> syn::Result<TokenStream> {
    match stmt {
        Stmt::Block(block) => sub_block(block),
        Stmt::IfElse(if_else) => sub_if_else(if_else),
        Stmt::Break(r#break) => sub_break(r#break),
        Stmt::Drop(drop) => sub_drop(drop),
        Stmt::Assign(assign) => sub_assign(assign),
    }
}

fn sub_block(_: Block) -> syn::Result<TokenStream> {
    todo!()
}

fn sub_if_else(_: IfElse) -> syn::Result<TokenStream> {
    todo!()
}

fn sub_break(_: Break) -> syn::Result<TokenStream> {
    todo!()
}

fn sub_drop(_: Drop) -> syn::Result<TokenStream> {
    todo!()
}

fn sub_assign(
    Assign { dst, eq_token: _, operation, semi_token: _ }: Assign,
) -> syn::Result<TokenStream> {
    let dst = sub_place(dst).unwrap_or_else(syn::Error::into_compile_error);
    let operation = sub_operation(operation).unwrap_or_else(syn::Error::into_compile_error);
    let dst_ident = internal_ident("dst");
    let operation_ident = internal_ident("operation");
    let builder_ident = builder_ident();
    Ok(quote! {
        {
            let #dst_ident = #dst;
            let #operation_ident = #operation;
            #builder_ident.add_operation_with_dst(#dst_ident, #operation_ident);
        }
    })
}

fn sub_place(place: Place) -> syn::Result<TokenStream> {
    match place {
        Place::Local(ident) => Ok(quote! { crate::mir::Place::clone(&#ident) }),
        Place::PlaceUse(ident) => {
            // in write_mir, a place_use simply names an existing Place.
            Ok(quote! { crate::mir::Place::clone(&#ident) })
        }
        Place::StaticField { .. } => match static_field_proj_suffix(place) {
            (base, Ok((ty, members))) => {
                let base = sub_place(base).unwrap_or_else(syn::Error::into_compile_error);
                Ok(quote! {
                    #base.proj(crate::mir::Projection::Field {
                        byte_offset: ::std::mem::offset_of!(#ty, #(#members).*),
                    })
                })
            }
            (_, Err(e)) => return Err(e),
        },
        Place::DynPtrMethod { receiver, type_having_dyn_ptr: dyn_ptr_type, .. } => {
            let receiver = sub_place(*receiver).unwrap_or_else(syn::Error::into_compile_error);
            let builder_ident = builder_ident();
            let receiver_ident = internal_ident("receiver");
            Ok(quote! {
                {
                    let #receiver_ident = #receiver;
                    <#dyn_ptr_type as crate::mir::reflection::HasDynPtr>::write_mir_get_data_ptr(
                        #builder_ident,
                        #receiver_ident,
                    )
                }
            })
        }
        Place::DynFieldMethod { receiver, expected_proj_type: _, arg, .. } => {
            let receiver = sub_place(*receiver).unwrap_or_else(syn::Error::into_compile_error);
            let syn::ExprConst { attrs: _, const_token: _, block } = arg;
            Ok(quote! {
                #receiver.proj(crate::mir::Projection::Field {
                    byte_offset: #block,
                })
            })
        }
        Place::Deref { receiver, .. } => {
            let receiver = sub_place(*receiver).unwrap_or_else(syn::Error::into_compile_error);
            Ok(quote! {
                #receiver.proj(crate::mir::Projection::Deref)
            })
        }
        Place::Index { receiver, index, .. } => {
            let receiver = sub_place(*receiver).unwrap_or_else(syn::Error::into_compile_error);
            Ok(quote! {
                #receiver.proj(crate::mir::Projection::Index { index: #index as usize })
            })
        }
        Place::Cast { receiver, expected_type, .. } => {
            let receiver = sub_place(*receiver).unwrap_or_else(syn::Error::into_compile_error);
            let builder_ident = builder_ident();
            Ok(quote! {
                {
                    assert!(#builder_ident.type_of_place(&#receiver).is::<#expected_type>());
                    #receiver
                }
            })
        }
    }
}

/// Returns the innermost place that is not a static field projection.
fn static_field_proj_suffix(place: Place) -> (Place, syn::Result<(syn::Type, Vec<syn::Member>)>) {
    match place {
        Place::StaticField { base, member, .. } => match static_field_proj_suffix(*base) {
            (base, Ok((outermost_ty, mut fields))) => {
                fields.push(member.clone());
                (base, Ok((outermost_ty, fields)))
            }
            (base, Err(mut e)) => {
                e.combine(syn::Error::new(
                    member.span(),
                    "Base type must be known for static field projection",
                ));
                (base, Err(e))
            }
        },
        Place::Deref { ref expected_type, .. } => {
            let ty = expected_type.clone();
            (place, Ok((ty, Vec::new())))
        }
        Place::DynFieldMethod { ref expected_proj_type, .. } => {
            let ty = expected_proj_type.clone();
            (place, Ok((ty, Vec::new())))
        }
        Place::Index { .. } => (
            place,
            Err(syn::Error::new(
                Span::call_site(),
                "Cannot use static field projection here. Don't know the base type.",
            )),
        ),
        Place::Cast { ref expected_type, .. } => {
            let ty = expected_type.clone();
            (place, Ok((ty, Vec::new())))
        }
        Place::DynPtrMethod { ref method, .. } => {
            let span = method.span();
            (
                place,
                Err(syn::Error::new(
                    span,
                    "Base type must be known for static field projection later",
                )),
            )
        }
        Place::PlaceUse(_ident) => todo!(),
        Place::Local(_ident) => todo!(),
    }
}

fn sub_operation(operation: Operation) -> syn::Result<TokenStream> {
    match operation {
        Operation::Operand(operand) => {
            let operand = sub_place_operand(operand).unwrap_or_else(syn::Error::into_compile_error);
            Ok(quote! {
                crate::mir::Operation::Operand(#operand)
            })
        }
        Operation::Const { value } => Ok(quote! {
            crate::mir::Operation::Const {
                value: crate::sim::value::BoxedAny::new(#value),
            }
        }),
        Operation::CallHostFunction { function, args } => {
            let args = args
                .into_iter()
                .map(|a| sub_place_operand(a).unwrap_or_else(syn::Error::into_compile_error));
            Ok(quote! {
                crate::mir::Operation::CallHostFunction {
                    function: &#function::FN_INFO,
                    args: ::std::vec![#(#args),*],
                }
            })
        }
    }
}

fn sub_place_operand(PlaceOperand { borrow, place }: PlaceOperand) -> syn::Result<TokenStream> {
    let place = sub_place(place).unwrap_or_else(syn::Error::into_compile_error);
    if let Some((_and_token, _mut_token)) = borrow {
        Ok(quote! {
            crate::mir::PlaceOperand::Borrow(#place)
        })
    } else {
        Ok(quote! {
            crate::mir::PlaceOperand::Move(#place)
        })
    }
}
