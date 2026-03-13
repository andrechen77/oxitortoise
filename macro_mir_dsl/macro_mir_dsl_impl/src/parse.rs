use syn::{
    Attribute, Block, FnArg, Generics, Ident, Pat, PatType, Token, Type, Visibility, braced,
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

pub struct MirIntrinsicSyntax {
    pub signature: MirIntrinsicSignatureSyntax,
    pub content: Block,
}

pub struct MirIntrinsicSignatureSyntax {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub ident: Ident,
    pub compile_time_generics: Generics,
    pub compile_time_args: Punctuated<FnArg, Token![,]>,
    pub runtime_generics: Generics,
    pub runtime_args: Punctuated<FnArg, Token![,]>,
    pub return_value_decl: PatType,
}

impl Parse for MirIntrinsicSyntax {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let signature = input.parse()?;
        let content;
        let brace_token = braced!(content in input);
        let stmts = content.call(Block::parse_within)?;

        Ok(MirIntrinsicSyntax { signature, content: Block { brace_token, stmts } })
    }
}

impl Parse for MirIntrinsicSignatureSyntax {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // parsing stolen from https://github.com/dtolnay/syn/blob/019a84847eded0cdb1f7856e0752ba618155cfc9/src/item.rs
        let attrs: Vec<Attribute> = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        let _fn_token: Option<Token![fn]> = input.parse()?;
        let ident: Ident = input.parse()?;
        let compile_time_generics: Generics = input.parse()?;
        let compile_time_args;
        let _compile_time_paren_token = parenthesized!(compile_time_args in input);
        let compile_time_args = parse_fn_args(&compile_time_args)?;
        let runtime_generics: Generics = input.parse()?;
        let runtime_args;
        let _runtime_paren_token = parenthesized!(runtime_args in input);
        let runtime_args = parse_fn_args(&runtime_args)?;
        let _arrow_token: Token![->] = input.parse()?;
        let return_place;
        let _return_paren_token = parenthesized!(return_place in input);
        let return_pat = Box::new(Pat::parse_single(&return_place)?);
        let return_colon_token: Token![:] = return_place.parse()?;
        let ty: Box<Type> = Box::new(return_place.parse()?);
        let return_value_decl =
            PatType { attrs: vec![], pat: return_pat, colon_token: return_colon_token, ty };

        Ok(MirIntrinsicSignatureSyntax {
            attrs,
            vis,
            ident,
            compile_time_generics,
            compile_time_args,
            runtime_generics,
            runtime_args,
            return_value_decl,
        })
    }
}

fn parse_fn_args(input: syn::parse::ParseStream) -> syn::Result<Punctuated<FnArg, Token![,]>> {
    let mut args = Punctuated::new();
    while !input.is_empty() {
        let attrs: Vec<Attribute> = input.call(Attribute::parse_outer)?;

        // parse a parameter
        let pat = Box::new(Pat::parse_single(input)?);
        let colon_token: Token![:] = input.parse()?;
        let ty: Box<Type> = Box::new(input.parse()?);
        args.push(FnArg::Typed(PatType { attrs, pat, colon_token, ty }));

        if input.is_empty() {
            break;
        }
        let comma: Token![,] = input.parse()?;
        args.push_punct(comma);
    }
    Ok(args)
}
