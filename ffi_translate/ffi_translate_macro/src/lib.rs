use proc_macro::TokenStream;

/// Thin wrapper around the proc_macro2 implementation
#[proc_macro_attribute]
pub fn ffi_translate(args: TokenStream, input: TokenStream) -> TokenStream {
    ffi_translate_impl::ffi_translate_impl(args.into(), input.into()).into()
}
