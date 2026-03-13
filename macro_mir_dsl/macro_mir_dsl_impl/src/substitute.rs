use proc_macro2::{Span, TokenStream};
use syn::{
    Block, ExprMacro, Macro,
    spanned::Spanned as _,
    visit_mut::{VisitMut, visit_expr_mut, visit_span_mut, visit_stmt_mut},
};

pub mod interp;
pub mod write_mir;

/// Substitutes inner instances of `mir!`, `type_of`, and `place_ref` in the given body,
/// using the given substitution functions.
fn substitute_internal<F, G, H>(
    mut body: Block,
    sub_mir_block: F,
    sub_type_of: G,
    sub_place_ref: H,
    return_expr: Option<TokenStream>,
    keep_spans: bool,
) -> syn::Result<Block>
where
    F: Fn(&Macro) -> syn::Result<(TokenStream, TokenStream)>,
    G: Fn(&ExprMacro) -> syn::Result<TokenStream>,
    H: Fn(&ExprMacro) -> syn::Result<TokenStream>,
{
    struct Visitor<F, G, H> {
        sub_mir_block: F,
        sub_type_of: G,
        sub_place_ref: H,
        hoisted: TokenStream,
        return_expr: Option<TokenStream>,
        keep_spans: bool,
    }

    impl<F, G, H> VisitMut for Visitor<F, G, H>
    where
        F: Fn(&Macro) -> syn::Result<(TokenStream, TokenStream)>,
        G: Fn(&ExprMacro) -> syn::Result<TokenStream>,
        H: Fn(&ExprMacro) -> syn::Result<TokenStream>,
    {
        fn visit_expr_return_mut(&mut self, r: &mut syn::ExprReturn) {
            // ensure that the return expression does not exist
            if let Some(expr) = &mut r.expr {
                *expr = Box::new(syn::Expr::Verbatim(
                    syn::Error::new(
                        expr.span(),
                        "do not return expressions; assign to the out local instead",
                    )
                    .into_compile_error(),
                ));
                return;
            }

            if let Some(return_expr) = &self.return_expr {
                r.expr = Some(Box::new(syn::Expr::Verbatim(return_expr.clone())));
            }
        }

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

        fn visit_span_mut(&mut self, s: &mut Span) {
            if self.keep_spans {
                *s = Span::call_site();
            } else {
                visit_span_mut(self, s);
            }
        }
    }

    let mut visitor = Visitor {
        sub_mir_block,
        sub_type_of,
        sub_place_ref,
        return_expr,
        hoisted: TokenStream::new(),
        keep_spans,
    };
    visitor.visit_block_mut(&mut body);

    body.stmts.insert(0, syn::Stmt::Expr(syn::Expr::Verbatim(visitor.hoisted), None));

    Ok(body)
}
