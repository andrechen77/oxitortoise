use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    ast::{
        Assign, Block, Break, Drop, IfElse, LocalDecl, MirBody, Operation, Place, PlaceOperand,
        Stmt, parse_mir_body, parse_place_ref, parse_type_of,
    },
    combine_generics, ensure_trailing,
    parse::{MirIntrinsicSignatureSyntax, MirIntrinsicSyntax},
    substitute::substitute_internal,
};

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
        if let syn::Type::Path(path) = pat_type.ty.as_ref()
            && let Some(ident) = path.path.get_ident()
            && ident == "any"
        {
            pat_type.ty = Box::new(syn::parse_quote! { crate::sim::value::BoxedAny });
        }
    }
    let runtime_args = ensure_trailing(runtime_args);
    let return_pat = return_value_decl.pat.clone();
    let return_ty = return_value_decl.ty.clone();

    let body = substitute_internal(
        &intrinsic.content,
        sub_mir_block,
        sub_type_of,
        sub_place_ref,
        Some(quote! { #return_pat }),
    )?;
    let body_stmts = body.stmts;

    // build the interp function
    let full_fn = quote! {
        #(#attrs)*
        #[allow(unused_braces)]
        #vis fn interp #generics(
            #compile_time_args
            #runtime_args
        ) -> #return_ty {
            let mut #return_value_decl;
            #(#body_stmts)*
            #return_pat
        }
    };
    Ok(full_fn)
}

fn sub_type_of(m: &syn::ExprMacro) -> syn::Result<TokenStream> {
    let inner_expr = parse_type_of(m)?;
    Ok(quote! { crate::sim::value::BoxedAny::ty(&#inner_expr) })
}

fn sub_place_ref(m: &syn::ExprMacro) -> syn::Result<TokenStream> {
    let inner_expr = parse_place_ref(m)?;
    // a "place ref" in the interp implementation is just a reference to the
    // place
    Ok(quote! { (&#inner_expr) })
}

/// Given the token stream input to a `mir!` macro, returns the code to replace
/// in the interp implementation. The first element of the tuple is code that
/// should be hoisted into the top of the function where the mir block appears.
/// The second element of the tuple is the code to replace the `mir!` macro with.
fn sub_mir_block(m: &syn::Macro) -> syn::Result<(TokenStream, TokenStream)> {
    let MirBody { local_decls, statements } = parse_mir_body(m)?;

    // map local declaration syntax
    let local_decls = local_decls
        .into_iter()
        .map(|local_decl| {
            let LocalDecl { let_token, name, colon_token, ty } = local_decl;
            quote! { #let_token mut #name #colon_token #ty; }
        })
        .collect();

    // map statement syntax
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

fn sub_block(Block { label, statements }: Block) -> syn::Result<TokenStream> {
    let statements: Vec<_> = statements
        .into_iter()
        .map(|statement| sub_stmt(statement).unwrap_or_else(syn::Error::into_compile_error))
        .collect();
    Ok(quote! {
        #label {
            #(#statements)*
        }
    })
}

fn sub_if_else(IfElse { if_token, condition, then, r#else }: IfElse) -> syn::Result<TokenStream> {
    let condition = sub_place(condition, false).unwrap_or_else(syn::Error::into_compile_error);
    let then = sub_stmt(*then).unwrap_or_else(syn::Error::into_compile_error);
    let r#else = r#else
        .map(|(else_token, else_stmt)| {
            let else_stmt = sub_stmt(*else_stmt).unwrap_or_else(syn::Error::into_compile_error);
            syn::Result::Ok(quote! {
                #else_token {
                    #else_stmt
                }
            })
        })
        .transpose()?;
    Ok(quote! {
        #if_token #condition {
            #then
        } #r#else
    })
}

fn sub_break(Break { target_label }: Break) -> syn::Result<TokenStream> {
    Ok(quote! {
        break #target_label;
    })
}

fn sub_drop(Drop { place }: Drop) -> syn::Result<TokenStream> {
    let place = sub_place(place, false).unwrap_or_else(syn::Error::into_compile_error);
    Ok(quote! {
        ::std::mem::drop(#place);
    })
}

fn sub_assign(Assign { dst, eq_token, operation, semi_token }: Assign) -> syn::Result<TokenStream> {
    // assigning to a place mutates it and thus requires a mutable borrow
    let dst = sub_place(dst, true).unwrap_or_else(syn::Error::into_compile_error);
    let operation = sub_operation(operation).unwrap_or_else(syn::Error::into_compile_error);
    Ok(quote! {
        #dst #eq_token #operation #semi_token
    })
}

fn sub_place(place: Place, can_mutate: bool) -> syn::Result<TokenStream> {
    match place {
        Place::Local(ident) => Ok(quote! { #ident }),
        Place::PlaceUse(ident) => {
            // a "place ref" in the interp implementation is just a reference,
            // so the usage would be the dereference
            Ok(quote! { (*#ident) })
        }
        Place::StaticField { base, dot_token, member } => {
            let base = sub_place(*base, can_mutate).unwrap_or_else(syn::Error::into_compile_error);
            Ok(quote! { #base #dot_token #member })
        }
        Place::DynPtrMethod { receiver, type_having_dyn_ptr: dyn_ptr_type, .. } => {
            let receiver =
                sub_place(*receiver, can_mutate).unwrap_or_else(syn::Error::into_compile_error);
            // Use the runtime HasDynPtr API; the surrounding module is
            // expected to have `HasDynPtr` in scope.
            Ok(quote! {
                <#dyn_ptr_type as crate::mir::reflection::HasDynPtr>::dyn_ptr_mut(&mut #receiver)
            })
        }
        Place::DynFieldMethod { receiver, expected_proj_type, arg, .. } => {
            let receiver =
                sub_place(*receiver, can_mutate).unwrap_or_else(syn::Error::into_compile_error);
            let syn::ExprConst { attrs: _, const_token: _, block: arg } = arg;
            // Interpret `const { expr }` as the runtime expression `{ expr }`
            // and project/cast through the dynamic pointer.
            Ok(quote! {
                *crate::mir::reflection::DynPtrMut::cast::<#expected_proj_type>(
                    crate::mir::reflection::DynPtrMut::proj_field(#receiver, #arg)
                )
            })
        }
        Place::Deref { receiver, .. } => {
            let receiver =
                sub_place(*receiver, can_mutate).unwrap_or_else(syn::Error::into_compile_error);
            Ok(quote! {
                (*#receiver)
            })
        }
        Place::Index { receiver, index, .. } => {
            let receiver =
                sub_place(*receiver, can_mutate).unwrap_or_else(syn::Error::into_compile_error);
            Ok(quote! {
                #receiver[#index]
            })
        }
        Place::Cast { receiver, expected_type, .. } => {
            let receiver =
                sub_place(*receiver, can_mutate).unwrap_or_else(syn::Error::into_compile_error);
            if can_mutate {
                Ok(quote! {
                    (*crate::sim::value::BoxedAny::cast_as_mut::<#expected_type>(&mut #receiver))
                })
            } else {
                Ok(quote! {
                    (*crate::sim::value::BoxedAny::cast_as::<#expected_type>(&#receiver))
                })
            }
        }
    }
}

fn sub_operation(operation: Operation) -> syn::Result<TokenStream> {
    match operation {
        Operation::Operand(operand) => sub_place_operand(operand),
        Operation::Const { value } => {
            // `const { expr }` in the DSL becomes just `{ expr }` at runtime.
            Ok(quote! { #value })
        }
        Operation::CallHostFunction { function, args } => {
            let args: Vec<_> =
                args.into_iter().map(sub_place_operand).collect::<syn::Result<_>>()?;
            Ok(quote! {
                #function::call (#(#args),*)
            })
        }
    }
}

fn sub_place_operand(PlaceOperand { borrow, place }: PlaceOperand) -> syn::Result<TokenStream> {
    let and_token = borrow.map(|(and_token, _)| and_token);
    let mut_token = borrow.and_then(|(_, mut_token)| mut_token);
    let place =
        sub_place(place, mut_token.is_some()).unwrap_or_else(syn::Error::into_compile_error);
    Ok(quote! {
        #and_token #mut_token #place
    })
}
