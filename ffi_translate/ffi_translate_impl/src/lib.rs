use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse2, FnArg, Ident, ItemFn, LitStr, Pat, ReturnType, Token, Type};

/// Trait for types that can be decomposed into FFI-safe components for passing
/// across FFI boundaries.
///
/// The `Components` associated type is always a tuple, even in the single arity
/// case. This type represents how an otherwise FFI-incompatible type can be
/// passed across an FFI boundary by being split into component FFI-safe types.
pub trait FfiCompose {
    /// The component types that this type can be decomposed into.
    /// This is always a tuple, even for single-arity cases (e.g., `(T,)`).
    type Components;

    /// Decomposes a value into its component parts.
    fn decompose(self) -> Self::Components;

    /// Recomposes component parts back into a value.
    ///
    /// This method takes the `Components` tuple as a single argument.
    /// The generated code from the `ffi_translate` macro may call this with
    /// the components as separate arguments (e.g., `recompose(a, b)`), which
    /// requires the tuple to be constructed implicitly or the trait to be
    /// implemented to support that call pattern.
    fn recompose(components: Self::Components) -> Self;
}

pub trait ComponentIndex<const N: usize> {
    type Component;
}

impl<T> ComponentIndex<0> for (T,) {
    type Component = T;
}

impl<T0, T1> ComponentIndex<0> for (T0, T1) {
    type Component = T0;
}

impl<T0, T1> ComponentIndex<1> for (T0, T1) {
    type Component = T1;
}

/// Implementation of the ffi_translate macro using proc_macro2 API
pub fn ffi_translate_impl(
    args: proc_macro2::TokenStream,
    input: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let input_fn = match parse2::<ItemFn>(input) {
        Ok(fn_item) => fn_item,
        Err(e) => return e.to_compile_error(),
    };

    let attr_args = match parse2::<FfiTranslateArgs>(args) {
        Ok(args) => args,
        Err(e) => return e.to_compile_error(),
    };

    let original_fn = &input_fn.sig.ident;
    let original_vis = &input_fn.vis;

    // filter out the ffi_translate attribute from the original function's attributes
    let filtered_attrs: Vec<_> = input_fn
        .attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("ffi_translate"))
        .cloned()
        .collect();

    // collect function arguments (excluding receiver)
    let fn_args_result = input_fn
        .sig
        .inputs
        .iter()
        .map(|input| match input {
            FnArg::Receiver(_) => Err(syn::Error::new_spanned(
                input,
                "ffi_translate does not support methods with receivers",
            )),
            FnArg::Typed(pat_type) => Ok(pat_type),
        })
        .collect::<Result<Vec<_>, _>>();
    let fn_args = match fn_args_result {
        Ok(fn_args) => fn_args,
        Err(err) => return err.to_compile_error(),
    };

    // validate that args_composition length matches the number of arguments
    if attr_args.args_composition.0.len() != fn_args.len() {
        return syn::Error::new_spanned(
            &attr_args.args_composition.1,
            format!(
                "args_composition must have {} elements (one for each argument), but found {}",
                fn_args.len(),
                attr_args.args_composition.0.len()
            ),
        )
        .to_compile_error();
    }

    // generate decomposed arguments and recompose statements
    let mut new_inputs: Vec<FnArg> = Vec::new();
    let mut new_inputs_types: Vec<Type> = Vec::new();
    let mut recompose_stmts = Vec::new();
    let mut original_args = Vec::new();

    for (pat_type, &arity) in fn_args.iter().zip(attr_args.args_composition.0.iter()) {
        let param_name = match &*pat_type.pat {
            Pat::Ident(ident) => &ident.ident,
            _ => {
                return syn::Error::new_spanned(
                    pat_type,
                    "ffi_translate requires identifier patterns for all arguments",
                )
                .to_compile_error();
            }
        };
        let param_ty = &pat_type.ty;

        // generate component arguments
        for i in 0..arity {
            let component_arg_name =
                Ident::new(&format!("{}_{}", param_name, i), param_name.span());
            let component_ty = quote! {
                <<#param_ty as Fc>::Components as Ci<#i>>::Component
            };
            new_inputs.push(parse2(quote! { #component_arg_name: #component_ty }).unwrap());
            new_inputs_types.push(parse2(component_ty).unwrap());
        }

        // generate recompose statement matching the example syntax
        let component_args: Vec<_> = (0..arity)
            .map(|i| {
                let component_arg_name =
                    Ident::new(&format!("{}_{}", param_name, i), param_name.span());
                quote! { #component_arg_name }
            })
            .collect();

        // match the example: recompose((a_0, a_1)) or recompose((b_0,))
        recompose_stmts.push(quote! {
            let #param_name = <#param_ty as Fc>::recompose((#(#component_args),*));
        });

        original_args.push(quote! { #param_name });
    }

    // build the new function
    let output = &input_fn.sig.output;
    let output_type: &[&Type] = match output {
        ReturnType::Type(_, ty) => &[ty],
        ReturnType::Default => &[],
    };
    let generics = &input_fn.sig.generics;
    let where_clause = &input_fn.sig.generics.where_clause;
    let module_name = original_fn.clone();
    let export_name = &attr_args.unsafe_export_name;
    let export_name_string = export_name.to_string();

    // generate the module with the FFI-compatible function
    let generated_module = quote! {
        mod #module_name {
            use super::*;

            use ::lir::{HostFunctionInfo, AsLir};
            use ::ffi_translate::FfiCompose as Fc;
            use ::ffi_translate::ComponentIndex as Ci;

            pub static INFO: HostFunctionInfo = HostFunctionInfo {
                name: #export_name_string,
                parameter_types: &[
                    #(<#new_inputs_types as AsLir>::STATIC_TYPE,)*
                ],
                return_type: &[#(<#output_type as AsLir>::STATIC_TYPE,)*],
            };

            #[unsafe(export_name = #export_name_string)]
            fn #export_name #generics(
                #(#new_inputs,)*
            ) #output
            #where_clause
            {
                #(#recompose_stmts)*
                super::#original_fn(#(#original_args),*)
            }
        }
    };

    // collect original inputs for expansion
    let original_inputs: Vec<_> = input_fn.sig.inputs.iter().cloned().collect();
    let block = &input_fn.block;

    // Return the original function (without ffi_translate attribute) and the generated module
    quote! {
        #(#filtered_attrs)*
        #original_vis fn #original_fn #generics(
            #(#original_inputs),*
        ) #output
        #where_clause
        #block

        #generated_module
    }
}

/// Parsed arguments for the ffi_translate attribute
struct FfiTranslateArgs {
    unsafe_export_name: Ident,
    args_composition: (Vec<usize>, Punctuated<syn::LitInt, Comma>),
}

impl Parse for FfiTranslateArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut unsafe_export_name = None;
        let mut args_composition = None;

        // Parse comma-separated arguments
        while !input.is_empty() {
            // Try to parse `unsafe_export_name = "name"`
            if input.peek(Ident) {
                let ident: Ident = input.parse()?;
                if ident == "unsafe_export_name" {
                    input.parse::<Token![=]>()?;
                    let lit: LitStr = input.parse()?;
                    unsafe_export_name = Some(Ident::new(&lit.value(), lit.span()));
                }
                // Try to parse `args_composition(1, 2, 3)`
                else if ident == "args_composition" {
                    let content;
                    syn::parenthesized!(content in input);
                    let args: Punctuated<syn::LitInt, Comma> =
                        content.parse_terminated(syn::LitInt::parse, Token![,])?;
                    args_composition = Some((
                        args.iter()
                            .map(|lit| lit.base10_parse::<usize>())
                            .collect::<Result<Vec<_>, _>>()?,
                        args,
                    ));
                } else {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("unknown argument: {}, expected 'unsafe_export_name' or 'args_composition'", ident),
                    ));
                }
            } else {
                break;
            }

            // Parse comma if not at end
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        let unsafe_export_name = unsafe_export_name
            .ok_or_else(|| syn::Error::new(input.span(), "unsafe_export_name is required"))?;

        let args_composition = args_composition
            .ok_or_else(|| syn::Error::new(input.span(), "args_composition is required"))?;

        Ok(FfiTranslateArgs { unsafe_export_name, args_composition })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn without_return_type() {
        let input = quote! {
            #[other_attributes]
            fn my_function(a: MyPair, b: MyPair) {
                do_something();
            }
        };

        let args = quote! {
            unsafe_export_name = "my_function_unmangled", args_composition(2, 2)
        };

        let output = ffi_translate_impl(args, input);

        // Try to parse the result as valid Rust code
        let _ = syn::parse2::<syn::File>(output.clone())
            .expect("generated code should parse successfully");

        let expected = quote! {
            #[other_attributes]
            fn my_function(a: MyPair, b: MyPair) {
                do_something();
            }
            mod my_function {
                use super::*;

                use ::lir::{HostFunctionInfo, AsLir};
                use ::ffi_translate::FfiCompose as Fc;
                use ::ffi_translate::ComponentIndex as Ci;

                pub static INFO: HostFunctionInfo = HostFunctionInfo {
                    name: "my_function_unmangled",
                    parameter_types: &[
                        < < <MyPair as Fc>::Components as Ci<0usize> >::Component as AsLir>::STATIC_TYPE,
                        < < <MyPair as Fc>::Components as Ci<1usize> >::Component as AsLir>::STATIC_TYPE,
                        < < <MyPair as Fc>::Components as Ci<0usize> >::Component as AsLir>::STATIC_TYPE,
                        < < <MyPair as Fc>::Components as Ci<1usize> >::Component as AsLir>::STATIC_TYPE,
                    ],
                    return_type: &[],
                };


                #[unsafe(export_name = "my_function_unmangled")]
                fn my_function_unmangled(
                    a_0: < <MyPair as Fc>::Components as Ci<0usize> >::Component,
                    a_1: < <MyPair as Fc>::Components as Ci<1usize> >::Component,
                    b_0: < <MyPair as Fc>::Components as Ci<0usize> >::Component,
                    b_1: < <MyPair as Fc>::Components as Ci<1usize> >::Component,
                ) {
                    let a = <MyPair as Fc>::recompose((a_0, a_1));
                    let b = <MyPair as Fc>::recompose((b_0, b_1));
                    super::my_function(a, b)
                }
            }
        };

        assert_eq!(output.to_string(), expected.to_string());
    }

    #[test]
    fn with_return_type() {
        let input = quote! {
            #[other_attributes]
            fn my_function(a: MyPair, b: MyPair) -> Thing {
                do_something()
            }
        };

        let args = quote! {
            unsafe_export_name = "my_function_unmangled", args_composition(2, 2)
        };

        let output = ffi_translate_impl(args, input);

        println!("output: {}", output.to_string());

        // Try to parse the result as valid Rust code
        let _ = syn::parse2::<syn::File>(output.clone())
            .expect("generated code should parse successfully");

        let expected = quote! {
            #[other_attributes]
            fn my_function(a: MyPair, b: MyPair) -> Thing {
                do_something()
            }
            mod my_function {
                use super::*;

                use ::lir::{HostFunctionInfo, AsLir};
                use ::ffi_translate::FfiCompose as Fc;
                use ::ffi_translate::ComponentIndex as Ci;

                pub static INFO: HostFunctionInfo = HostFunctionInfo {
                    name: "my_function_unmangled",
                    parameter_types: &[
                        < < <MyPair as Fc>::Components as Ci<0usize> >::Component as AsLir>::STATIC_TYPE,
                        < < <MyPair as Fc>::Components as Ci<1usize> >::Component as AsLir>::STATIC_TYPE,
                        < < <MyPair as Fc>::Components as Ci<0usize> >::Component as AsLir>::STATIC_TYPE,
                        < < <MyPair as Fc>::Components as Ci<1usize> >::Component as AsLir>::STATIC_TYPE,
                    ],
                    return_type: &[<Thing as AsLir>::STATIC_TYPE,],
                };


                #[unsafe(export_name = "my_function_unmangled")]
                fn my_function_unmangled(
                    a_0: < <MyPair as Fc>::Components as Ci<0usize> >::Component,
                    a_1: < <MyPair as Fc>::Components as Ci<1usize> >::Component,
                    b_0: < <MyPair as Fc>::Components as Ci<0usize> >::Component,
                    b_1: < <MyPair as Fc>::Components as Ci<1usize> >::Component,
                ) -> Thing {
                    let a = <MyPair as Fc>::recompose((a_0, a_1));
                    let b = <MyPair as Fc>::recompose((b_0, b_1));
                    super::my_function(a, b)
                }
            }
        };

        assert_eq!(output.to_string(), expected.to_string());
    }
}
