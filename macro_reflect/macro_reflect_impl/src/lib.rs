use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Data, DeriveInput, Fields, Ident, ItemImpl, Meta, parse::Parse, parse2, punctuated::Punctuated,
    token::Comma,
};

/// Attributes parsed from `#[reflect(...)]`.
struct ReflectArgs {
    is_zeroable: bool,
    no_drop: bool,
    clone_kind: CloneKind,
    special_mir_type: bool,
    primitive_type: Option<TokenStream>,
}

enum CloneKind {
    Copy,
    Dynamic,
    None,
}

impl Default for ReflectArgs {
    fn default() -> Self {
        Self {
            is_zeroable: false,
            clone_kind: CloneKind::None,
            no_drop: false,
            special_mir_type: false,
            primitive_type: None,
        }
    }
}

impl Parse for ReflectArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut out = ReflectArgs::default();
        for meta in Punctuated::<Meta, Comma>::parse_terminated(input)? {
            match &meta {
                Meta::List(list) if list.path.is_ident("unsafe") => {
                    let _ = list.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated).map(
                        |inner| {
                            for m in inner {
                                if let Meta::Path(p) = &m
                                    && p.is_ident("is_zeroable")
                                {
                                    out.is_zeroable = true;
                                } else if let Meta::List(list) = &m
                                    && list.path.is_ident("primitive")
                                {
                                    out.primitive_type = Some(list.tokens.clone());
                                }
                            }
                        },
                    );
                }
                Meta::List(list) if list.path.is_ident("clone") => {
                    let _ = list.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated).map(
                        |inner| {
                            for m in inner {
                                if let Meta::Path(p) = &m {
                                    if p.is_ident("copy") {
                                        out.clone_kind = CloneKind::Copy;
                                    } else if p.is_ident("dynamic") {
                                        out.clone_kind = CloneKind::Dynamic;
                                    } else if p.is_ident("none") {
                                        out.clone_kind = CloneKind::None;
                                    }
                                }
                            }
                        },
                    );
                }
                Meta::Path(p) if p.is_ident("no_drop") => {
                    out.no_drop = true;
                }
                Meta::Path(p) if p.is_ident("special_mir_type") => {
                    out.special_mir_type = true;
                }
                other => {
                    return Err(syn::Error::new_spanned(other, "unexpected attribute"));
                }
            }
        }
        Ok(out)
    }
}

fn get_reflection_crate_name() -> proc_macro2::TokenStream {
    match crate_name("oxitortoise_reflection").unwrap() {
        FoundCrate::Itself => quote! { crate },
        FoundCrate::Name(name) => {
            let name = Ident::new(&name, Span::call_site());
            quote! { ::#name }
        }
    }
}

pub fn derive_impl_mir_reflect(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let reflection_crate_name = get_reflection_crate_name();

    let input = match parse2::<DeriveInput>(input) {
        Ok(parsed) => parsed,
        Err(e) => return e.to_compile_error(),
    };

    let name = &input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let make_dyn_type_fn = match &input.data {
        Data::Struct(data) => {
            // mark fields that have the attribute #[mir_accessible]
            let fields = match &data.fields {
                Fields::Unit => vec![],
                Fields::Named(n) => n.named.iter().collect(),
                Fields::Unnamed(u) => u.unnamed.iter().collect(),
            };
            let mut mir_accessible_field_entries = Vec::new();
            let mut _is_exhaustive = true; // can use this if deciding whether the struct is complete
            for field in fields {
                if let Some(attr) =
                    field.attrs.iter().find(|attr| attr.meta.path().is_ident("mir_accessible"))
                {
                    let unchecked = if let Meta::List(list) = &attr.meta {
                        match list.parse_args::<Ident>() {
                            Ok(ident) if ident == "unchecked_type" => {}
                            Ok(_) => {
                                return syn::Error::new_spanned(attr, "expected `unchecked_type`")
                                    .to_compile_error();
                            }
                            Err(e) => return e.to_compile_error(),
                        }
                        true
                    } else {
                        false
                    };
                    let ident = field.ident.as_ref().unwrap();
                    let ty = &field.ty;
                    let field_entry = if unchecked {
                        quote! {
                            (::std::mem::offset_of!(Self, #ident), #reflection_crate_name::DynType::default())
                        }
                    } else {
                        quote! {
                            (::std::mem::offset_of!(Self, #ident), <#ty as #reflection_crate_name::Reflect>::dyn_type())
                        }
                    };
                    mir_accessible_field_entries.push(field_entry);
                } else {
                    _is_exhaustive = false;
                }
            }

            quote! {
                fn create_dyn_type() -> #reflection_crate_name::DynType {
                    #reflection_crate_name::DynType::new_struct_with_static_type::<Self>(
                        vec![#(#mir_accessible_field_entries),*],
                    )
                }
            }
        }
        _ => {
            return syn::Error::new_spanned(
                &input,
                "Reflect derive macro currently only supports structs",
            )
            .to_compile_error();
        }
    };

    // add a bound to the where clause that the type implements the Reflect trait
    let where_clause = if let Some(where_clause) = where_clause {
        quote! {
            #where_clause, Self: #reflection_crate_name::Reflect
        }
    } else {
        quote! {
            where Self: #reflection_crate_name::Reflect
        }
    };

    quote! {
        #[automatically_derived]
        unsafe impl #impl_generics #reflection_crate_name::CreateDynType for #name #ty_generics #where_clause {
            #make_dyn_type_fn
        }
    }
}

pub fn attribute_impl_reflect(args: TokenStream, input: TokenStream) -> TokenStream {
    let reflection_crate_name = get_reflection_crate_name();

    let input: ItemImpl = match parse2(input) {
        Ok(parsed) => parsed,
        Err(e) => return e.to_compile_error(),
    };

    // make sure that the trait is Reflect
    if let Some((None, trait_name, _for_token)) = input.trait_
        && trait_name.is_ident("Reflect")
    {
        // this is good
    } else {
        // this is bad
        return syn::Error::new_spanned(
            input.impl_token,
            "Reflect attribute must be applied to an impl of the `Reflect` (verbatim) trait",
        )
        .to_compile_error();
    };

    let attrs: ReflectArgs = match parse2(args) {
        Ok(parsed) => parsed,
        Err(e) => return e.to_compile_error(),
    };

    let self_ty = &input.self_ty;

    // generate a module to hold the static variables and stuff
    let mod_name =
        internal_name(&format!("mod_{}", sanitize_ident(&self_ty.to_token_stream().to_string())));

    let mut clone_fn_info_def = None;
    let clone_fn = match attrs.clone_kind {
        CloneKind::Copy => quote! {
            #reflection_crate_name::CloneKind::Copy
        },
        CloneKind::Dynamic => {
            let clone_fn_name = format!("{}::clone", self_ty.to_token_stream().to_string());
            clone_fn_info_def = Some(quote! {
                static CLONE_HOST_FN_INFO: #reflection_crate_name::mir::HostFunctionInfo = #reflection_crate_name::mir::HostFunctionInfo {
                    debug_name: #clone_fn_name,
                    parameter_types: &[&<&#self_ty as #reflection_crate_name::Reflect>::STATIC_TYPE],
                    return_type: &TYPE_INFO,
                    link_name: #clone_fn_name,
                    link_addr: clone as *const u8,
                };

                #[unsafe(export_name = #clone_fn_name)]
                fn clone(value: &#self_ty) -> #self_ty {
                    value.clone()
                }
            });
            quote! {
                #reflection_crate_name::CloneKind::Dynamic {
                    clone_fn_info: &CLONE_HOST_FN_INFO,
                }
            }
        }
        CloneKind::None => quote! {
            #reflection_crate_name::CloneKind::None
        },
    };

    let drop_fn = if attrs.no_drop {
        quote! { None }
    } else {
        quote! { Some(|ptr| {
            #[allow(unused_unsafe)]
            unsafe { ::std::ptr::drop_in_place(ptr as *mut #self_ty) };
        }) }
    };

    let is_zeroable = attrs.is_zeroable;

    let dyn_type_static_var = quote! {
        static DYN_TYPE: ::std::sync::LazyLock<#reflection_crate_name::DynType> = ::std::sync::LazyLock::new(|| {
            <#self_ty as #reflection_crate_name::CreateDynType>::create_dyn_type()
        });
    };

    let primitive_type = if let Some(primitive_type) = attrs.primitive_type {
        quote! { Some(#primitive_type) }
    } else {
        quote! { None }
    };

    let type_info_def = quote! {
        static TYPE_INFO: #reflection_crate_name::StaticTypeInfo = #reflection_crate_name::StaticTypeInfo {
            debug_name: stringify!(#self_ty),
            layout: Some(::std::alloc::Layout::new::<#self_ty>()),
            is_zeroable: #is_zeroable,
            clone: #clone_fn,
            drop_fn: #drop_fn,
            dyn_type: &DYN_TYPE,
            primitive_type: #primitive_type
        };
    };

    quote! {
        #[automatically_derived]
        mod #mod_name {
            use super::*;

            #dyn_type_static_var

            #type_info_def

            #clone_fn_info_def

            unsafe impl #reflection_crate_name::Reflect for #self_ty {
                const STATIC_TYPE: #reflection_crate_name::StaticType = &TYPE_INFO;

                const DYN_TYPE: &'static ::std::sync::LazyLock<#reflection_crate_name::DynType> = &DYN_TYPE;
            }
        }
    }
}

fn sanitize_ident(str: &str) -> String {
    fn is_ident_start(c: char) -> bool {
        c == '_' || c.is_ascii_alphabetic()
    }

    fn is_ident_continue(c: char) -> bool {
        c == '_' || c.is_ascii_alphanumeric()
    }

    str.char_indices()
        .filter_map(|(i, c)| {
            if (i == 0 && is_ident_start(c)) || (i > 0 && is_ident_continue(c)) {
                Some(c)
            } else if c.is_whitespace() {
                None
            } else {
                Some('_')
            }
        })
        .collect()
}

fn internal_name(name: &str) -> Ident {
    Ident::new(&format!("__internal_macro_reflect_{}", name), Span::mixed_site())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_impl_components() {
        let input = quote! {
            struct Test<T: Bound> {
                a: u32,
                b: u32,
            }
        };
        let output = derive_impl_mir_reflect(input);
        println!("output: {}", output.to_string());
        let _ = syn::parse2::<syn::File>(output.clone())
            .expect("generated code should parse successfully");
    }

    #[test]
    fn test_attribute_impl_reflect() {
        let args = quote! { clone(copy) };
        let input = quote! {
            impl Reflect for Test<T> {}
        };
        let output = attribute_impl_reflect(args, input);
        println!("output: {}", output.to_string());
        let _ = syn::parse2::<syn::File>(output.clone())
            .expect("generated code should parse successfully");
    }

    #[test]
    fn test_attribute_impl_reflect_clone_dynamic() {
        let args = quote! { clone(dynamic) };
        let input = quote! {
            impl Reflect for Test<T> {}
        };
        let output = attribute_impl_reflect(args, input);
        println!("output: {}", output.to_string());
        let _ = syn::parse2::<syn::File>(output.clone())
            .expect("generated code should parse successfully");
    }

    #[test]
    fn test_attribute_impl_reflect_invalid_trait() {
        let args = quote! { clone(copy) };
        let input = quote! {
            impl Refloct for Test<T> {}
        };
        let output = attribute_impl_reflect(args, input);
        println!("output: {}", output.to_string());
    }
}
