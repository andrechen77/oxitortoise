use proc_macro::TokenStream;

/// Thin wrapper around the proc_macro2 implementation
#[proc_macro]
pub fn mir_intrinsic(input: TokenStream) -> TokenStream {
    oxitortoise_macro_mir_dsl_impl::mir_intrinsic_impl(input.into()).into()
}
