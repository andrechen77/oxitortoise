use proc_macro::TokenStream;

/// Derive macro. Delegates to the proc_macro2 implementation.
#[proc_macro_derive(ReflectComponents, attributes(mir_accessible))]
pub fn reflect_components(input: TokenStream) -> TokenStream {
    macro_reflect_impl::derive_impl_components(input.into()).into()
}

#[proc_macro_attribute]
pub fn reflect(args: TokenStream, input: TokenStream) -> TokenStream {
    macro_reflect_impl::attribute_impl_reflect(args.into(), input.into()).into()
}
