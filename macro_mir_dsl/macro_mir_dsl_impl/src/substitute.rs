use proc_macro2::TokenStream;
use syn::{
    Block, ExprMacro, Macro,
    visit_mut::{VisitMut, visit_expr_mut, visit_stmt_mut},
};

pub mod interp;
pub mod write_mir;

/// Substitutes inner instances of `mir!`, `type_of`, and `place_ref` in the given body,
/// using the given substitution functions.
fn substitute_internal<F, G, H, I>(
    body: &Block,
    sub_mir_block: F,
    sub_type_of: G,
    sub_place_ref: H,
    sub_type_mapping: I,
) -> syn::Result<Block>
where
    F: Fn(&Macro) -> syn::Result<(TokenStream, TokenStream)>,
    G: Fn(&ExprMacro) -> syn::Result<TokenStream>,
    H: Fn(&ExprMacro) -> syn::Result<TokenStream>,
    I: Fn(&ExprMacro) -> syn::Result<TokenStream>,
{
    struct Visitor<F, G, H, I> {
        sub_mir_block: F,
        sub_type_of: G,
        sub_place_ref: H,
        sub_type_mapping: I,
        hoisted: TokenStream,
    }

    impl<F, G, H, I> VisitMut for Visitor<F, G, H, I>
    where
        F: Fn(&Macro) -> syn::Result<(TokenStream, TokenStream)>,
        G: Fn(&ExprMacro) -> syn::Result<TokenStream>,
        H: Fn(&ExprMacro) -> syn::Result<TokenStream>,
        I: Fn(&ExprMacro) -> syn::Result<TokenStream>,
    {
        fn visit_stmt_mut(&mut self, s: &mut syn::Stmt) {
            if let syn::Stmt::Macro(m) = s
                && let Some(ident) = m.mac.path.get_ident()
                && ident == "mir"
            {
                let (hoisted, inline) = match (self.sub_mir_block)(&m.mac) {
                    Ok(interp_sub) => interp_sub,
                    Err(e) => (TokenStream::new(), e.into_compile_error()),
                };

                self.hoisted.extend(hoisted);

                let mut expr = syn::parse2(inline).expect("macro output should parse");
                // continue visitation in case there is more stuff in the const blocks
                visit_expr_mut(self, &mut expr);

                *s = syn::Stmt::Expr(expr, None);
            } else {
                visit_stmt_mut(self, s);
            }
        }

        fn visit_expr_mut(&mut self, e: &mut syn::Expr) {
            if let syn::Expr::Macro(m) = e
                && let Some(ident) = m.mac.path.get_ident()
            {
                if ident == "type_of" {
                    let substitution =
                        (self.sub_type_of)(m).unwrap_or_else(syn::Error::into_compile_error);
                    *e = syn::Expr::Verbatim(substitution);
                } else if ident == "place_ref" {
                    let substitution =
                        (self.sub_place_ref)(m).unwrap_or_else(syn::Error::into_compile_error);
                    *e = syn::Expr::Verbatim(substitution);
                } else if ident == "type_mapping" {
                    let substitution =
                        (self.sub_type_mapping)(m).unwrap_or_else(syn::Error::into_compile_error);
                    *e = syn::Expr::Verbatim(substitution);
                } else if ident == "mir" {
                    let (hoisted, inline) = match (self.sub_mir_block)(&m.mac) {
                        Ok(interp_sub) => interp_sub,
                        Err(e) => (TokenStream::new(), e.into_compile_error()),
                    };
                    self.hoisted.extend(hoisted);

                    *e = syn::parse2(inline).expect("macro output should parse");

                    // continue visitation in case there is more stuff in the const blocks
                    visit_expr_mut(self, e);
                } else {
                    visit_expr_mut(self, e);
                }
            } else {
                visit_expr_mut(self, e);
            }
        }
    }

    let mut visitor = Visitor {
        sub_mir_block,
        sub_type_of,
        sub_place_ref,
        sub_type_mapping,
        hoisted: TokenStream::new(),
    };
    let mut body = body.clone();
    visitor.visit_block_mut(&mut body);

    body.stmts.insert(0, syn::Stmt::Expr(syn::Expr::Verbatim(visitor.hoisted), None));

    Ok(body)
}
