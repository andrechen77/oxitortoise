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

pub fn derive_impl_mir_reflect(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let input = match parse2::<DeriveInput>(input) {
        Ok(parsed) => parsed,
        Err(e) => return e.to_compile_error(),
    };

    let name = &input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let make_mir_type_fn = match &input.data {
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
                            (::std::mem::offset_of!(Self, #ident), crate::mir::MirType::default())
                        }
                    } else {
                        quote! {
                            (::std::mem::offset_of!(Self, #ident), <#ty as crate::util::reflection::Reflect>::mir_type())
                        }
                    };
                    mir_accessible_field_entries.push(field_entry);
                } else {
                    _is_exhaustive = false;
                }
            }

            quote! {
                fn create_mir_type() -> crate::mir::MirType {
                    crate::mir::MirType::new_struct_with_static_type::<Self>(
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
            #where_clause, Self: crate::util::reflection::Reflect
        }
    } else {
        quote! {
            where Self: crate::util::reflection::Reflect
        }
    };

    quote! {
        #[automatically_derived]
        unsafe impl #impl_generics crate::mir::MirReflect for #name #ty_generics #where_clause {
            #make_mir_type_fn
        }
    }
}

pub fn attribute_impl_reflect(args: TokenStream, input: TokenStream) -> TokenStream {
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
            crate::util::reflection::CloneKind::Copy
        },
        CloneKind::Dynamic => {
            let clone_fn_name = format!("{}::clone", self_ty.to_token_stream().to_string());
            clone_fn_info_def = Some(quote! {
                static CLONE_HOST_FN_INFO: crate::mir::HostFunctionInfo = crate::mir::HostFunctionInfo {
                    debug_name: #clone_fn_name,
                    parameter_types: &[&<&#self_ty as crate::util::reflection::Reflect>::TYPE],
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
                crate::util::reflection::CloneKind::Dynamic {
                    clone_fn_info: &CLONE_HOST_FN_INFO,
                }
            }
        }
        CloneKind::None => quote! {
            crate::util::reflection::CloneKind::None
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

    let mir_type_static = quote! {
        static MIR_TYPE: ::std::sync::LazyLock<crate::mir::MirType> = ::std::sync::LazyLock::new(|| {
            <#self_ty as crate::mir::MirReflect>::create_mir_type()
        });
    };

    let type_info_def = quote! {
        static TYPE_INFO: crate::util::reflection::TypeInfo = crate::util::reflection::TypeInfo {
            debug_name: stringify!(#self_ty),
            layout: Some(::std::alloc::Layout::new::<#self_ty>()),
            is_zeroable: #is_zeroable,
            clone: #clone_fn,
            drop_fn: #drop_fn,
            mir_type: &MIR_TYPE,
        };
    };

    quote! {
        #[automatically_derived]
        mod #mod_name {
            use super::*;

            #mir_type_static

            #type_info_def

            #clone_fn_info_def

            unsafe impl crate::util::reflection::Reflect for #self_ty {
                const TYPE: crate::util::reflection::Type = &TYPE_INFO;

                const MIR_TYPE: &'static ::std::sync::LazyLock<crate::mir::MirType> = &MIR_TYPE;
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
