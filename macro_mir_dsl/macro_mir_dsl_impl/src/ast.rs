//! An abstract syntax-focused parallel of the MIR defined in `engine/src/mir.rs`.

use quote::ToTokens;
use syn::{
    AngleBracketedGenericArguments, BinOp, Expr, ExprAssign, ExprBinary, ExprCall, ExprConst,
    ExprField, ExprMacro, ExprMethodCall, ExprParen, ExprPath, ExprReference, ExprUnary,
    GenericArgument, Ident, Label, Local, LocalInit, Member, Pat, PatIdent, Path, Token, Type,
    UnOp,
    parse::{Parse, ParseStream},
    parse2,
    spanned::Spanned,
};

pub struct MirBody {
    pub local_decls: Vec<LocalDecl>,
    pub statements: Vec<Stmt>,
}

pub struct LocalDecl {
    pub let_token: Token![let],
    pub name: Ident,
    pub colon_token: Token![:],
    pub ty: Type,
}

pub enum Stmt {
    Block(Block),
    IfElse(IfElse),
    Break(Break),
    Drop(Drop),
    Assign(Assign),
}

pub struct Block {
    pub label: Label,
    pub statements: Vec<Stmt>,
}

pub struct IfElse {
    pub if_token: Token![if],
    pub condition: Place,
    pub then: Box<Stmt>,
    pub r#else: Option<(Token![else], Box<Stmt>)>,
}

pub struct Break {
    pub target_label: Label,
}

pub struct Drop {
    pub place: Place,
}

pub struct Assign {
    pub dst: Place,
    pub eq_token: Token![=],
    pub operation: Operation,
    pub semi_token: Token![;],
}

pub enum Place {
    /// `place_use!(name)`
    PlaceUse(Ident),
    /// `local_name`
    Local(Ident),
    /// `base.member`
    StaticField { base: Box<Place>, dot_token: Token![.], member: Member },
    /// `receiver.dyn_ptr::<TypeHavingDynPtr>()`
    DynPtrMethod {
        receiver: Box<Place>,
        /// always `dyn_ptr`
        method: Ident,
        type_having_dyn_ptr: Type,
    },
    /// `receiver.dyn_field::<ExpectedProjType>(const { expr })`
    DynFieldMethod {
        receiver: Box<Place>,
        // allow this because we might want to use it later if we need a attache
        // a span to an error related to this field
        #[allow(dead_code)]
        /// always `dyn_field`
        method: Ident,
        expected_proj_type: Type,
        arg: ExprConst,
    },
    /// `receiver.deref::<DerefType>()`
    Deref {
        receiver: Box<Place>,
        // allow this because we might want to use it later if we need a attache
        // a span to an error related to this field
        #[allow(dead_code)]
        /// always `deref`
        method: Ident,
        expected_type: Type,
    },
    /// `receiver.index(index)`
    Index {
        receiver: Box<Place>,
        // allow this because we might want to use it later if we need a attache
        // a span to an error related to this field
        #[allow(dead_code)]
        /// always `index`
        method: Ident,
        index: Ident,
    },
    /// `receiver.cast::<Type>()`
    Cast {
        receiver: Box<Place>,
        // allow this because we might want to use it later if we need a attache
        // a span to an error related to this field
        #[allow(dead_code)]
        /// always `cast`
        method: Ident,
        expected_type: Type,
    },
}

pub enum Operation {
    Operand(PlaceOperand),
    Const { value: syn::Block },
    BinaryOp { op: BinOp, lhs: PlaceOperand, rhs: PlaceOperand },
    UnaryOp { op: UnOp, operand: PlaceOperand },
    CallHostFunction { function: Path, args: Vec<PlaceOperand> },
}

pub struct PlaceOperand {
    pub borrow: Option<(Token![&], Option<Token![mut]>)>,
    pub place: Place,
}

macro_rules! no_attrs {
    ($expr:expr) => {
        if !$expr.is_empty() {
            return Err(syn::Error::new(
                $expr.first().unwrap().span(),
                "Attributes are not allowed here",
            ));
        }
    };
}

pub fn parse_mir_body(m: &syn::Macro) -> syn::Result<MirBody> {
    // enforce path is "mir"
    assert!(m.path.get_ident().unwrap() == "mir");

    // now we know the inside can be parsed as a block of MIR DSL
    struct VecStmts(Vec<syn::Stmt>);
    impl Parse for VecStmts {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            syn::Block::parse_within(input).map(VecStmts)
        }
    }
    let VecStmts(stmts) = parse2(m.tokens.clone())?;

    let mut local_decls = Vec::new();
    let mut mir_stmts = Vec::new();
    for stmt in stmts {
        println!("parsing mir stmt: {}", stmt.to_token_stream().to_string());
        let (mir_stmt, local_decl) = parse_mir_stmt(stmt)?;
        local_decls.extend(local_decl);
        mir_stmts.extend(mir_stmt);
    }

    Ok(MirBody { local_decls, statements: mir_stmts })
}

pub fn parse_mir_stmt(stmt: syn::Stmt) -> syn::Result<(Option<Stmt>, Option<LocalDecl>)> {
    let stmt = match stmt {
        syn::Stmt::Local(Local { attrs, let_token, pat, init, semi_token }) => {
            no_attrs!(attrs);
            let Pat::Type(pat_type) = pat else {
                return Err(syn::Error::new(
                    pat.span(),
                    "Place declarations must be typed, as in `let name: Type`",
                ));
            };
            let Pat::Ident(PatIdent { attrs, by_ref: None, mutability: None, ident, subpat: None }) =
                *pat_type.pat
            else {
                return Err(syn::Error::new(
                    pat_type.pat.span(),
                    "Place declarations must use a simple identifier pattern, as in `let name: Type`",
                ));
            };
            no_attrs!(attrs);

            let stmt = if let Some(local_init) = init {
                let LocalInit { diverge: None, eq_token, expr } = local_init else {
                    return Err(syn::Error::new(
                        local_init.expr.span(),
                        "Place declarations cannot have divergent initializers",
                    ));
                };

                let dst = Place::Local(ident.clone());
                let operation = expr_to_operation(*expr)?;
                Some(Stmt::Assign(Assign { dst, eq_token, operation, semi_token }))
            } else {
                None
            };

            let mir_decl = LocalDecl {
                let_token,
                name: ident.clone(),
                colon_token: pat_type.colon_token,
                ty: *pat_type.ty,
            };

            return Ok((stmt, Some(mir_decl)));
        }
        syn::Stmt::Expr(Expr::Assign(assign), semi_token) => {
            let Some(semi_token) = semi_token else {
                return Err(syn::Error::new(
                    assign.span(),
                    "Assignment statements must end with a semicolon",
                ));
            };
            let ExprAssign { attrs, left, eq_token, right } = assign;
            no_attrs!(attrs);

            let dst = expr_to_place(*left)?;
            let operation = expr_to_operation(*right)?;
            Stmt::Assign(Assign { dst, eq_token, operation, semi_token })
        }
        syn::Stmt::Expr(Expr::Call(call), semicolon_token) => {
            if semicolon_token.is_none() {
                return Err(syn::Error::new(call.span(), "Statements must end with a semicolon"));
            }
            let ExprCall { attrs, func, paren_token: _, args } = call;
            no_attrs!(attrs);

            // check if this is a call to drop function
            let func_name = expr_path_to_ident(*func)?;
            if func_name == "drop" {
                // enforce single argument
                if args.len() != 1 {
                    return Err(syn::Error::new(
                        args.span(),
                        "The dyn_field method requires exactly one argument, the constant expression representing the field to project to",
                    ));
                }
                let arg = args.into_iter().next().unwrap();
                let place = expr_to_place(arg)?;
                Stmt::Drop(Drop { place })
            } else {
                return Err(syn::Error::new(func_name.span(), "unrecognized call syntax"));
            }
        }
        _ => todo!(),
    };
    Ok((Some(stmt), None))
}

fn expr_to_place(expr: Expr) -> syn::Result<Place> {
    match expr {
        Expr::Paren(ExprParen { attrs, expr, paren_token: _ }) => {
            no_attrs!(attrs);
            expr_to_place(*expr)
        }
        expr_path @ Expr::Path(_) => {
            let ident = expr_path_to_ident(expr_path)?;
            Ok(Place::Local(ident))
        }
        Expr::Field(ExprField { attrs, base, dot_token, member }) => {
            no_attrs!(attrs);
            let base = Box::new(expr_to_place(*base)?);
            Ok(Place::StaticField { base, dot_token, member })
        }
        Expr::MethodCall(ExprMethodCall { attrs, receiver, method, turbofish, args, .. }) => {
            no_attrs!(attrs);

            // parse receiver as a place
            let receiver = Box::new(expr_to_place(*receiver)?);

            // match on kind of method call
            match method.to_string().as_str() {
                "dyn_ptr" => {
                    // enforce turbofish exists with exactly one type argument
                    let Some(AngleBracketedGenericArguments { args: generic_args, .. }) = turbofish
                    else {
                        return Err(syn::Error::new(
                            turbofish.span(),
                            "The special dyn_field method requires a type annotation for the expected projected type",
                        ));
                    };
                    if generic_args.len() != 1 {
                        return Err(syn::Error::new(
                            generic_args.span(),
                            "The dyn_field method requires exactly one generic argument, the type annotation for the expected projected type",
                        ));
                    }
                    let generic_arg = generic_args.into_iter().next().unwrap();
                    let GenericArgument::Type(ty) = generic_arg else {
                        return Err(syn::Error::new(
                            generic_arg.span(),
                            "The generic argument for the expected projected type must be a type",
                        ));
                    };

                    // enforce no arguments
                    if args.len() != 0 {
                        return Err(syn::Error::new(
                            args.span(),
                            "The dyn_ptr method does not accept any arguments",
                        ));
                    }

                    Ok(Place::DynPtrMethod { receiver, method, type_having_dyn_ptr: ty })
                }
                "dyn_field" => {
                    // enforce turbofish exists with exactly one type argument
                    let Some(AngleBracketedGenericArguments { args: generic_args, .. }) = turbofish
                    else {
                        return Err(syn::Error::new(
                            turbofish.span(),
                            "The special dyn_field method requires a type annotation for the expected projected type",
                        ));
                    };
                    if generic_args.len() != 1 {
                        return Err(syn::Error::new(
                            generic_args.span(),
                            "The dyn_field method requires exactly one generic argument, the type annotation for the expected projected type",
                        ));
                    }
                    let generic_arg = generic_args.into_iter().next().unwrap();
                    let GenericArgument::Type(ty) = generic_arg else {
                        return Err(syn::Error::new(
                            generic_arg.span(),
                            "The generic argument for the expected projected type must be a type",
                        ));
                    };

                    // enforce argument is exactly one constant expression
                    if args.len() != 1 {
                        return Err(syn::Error::new(
                            args.span(),
                            "The dyn_field method requires exactly one argument, the constant expression representing the field to project to",
                        ));
                    }
                    let arg = args.into_iter().next().unwrap();
                    let Expr::Const(arg) = arg else {
                        return Err(syn::Error::new(
                            arg.span(),
                            "The argument to the dyn_field method must be a constant expression",
                        ));
                    };

                    Ok(Place::DynFieldMethod { receiver, method, expected_proj_type: ty, arg })
                }
                "deref" => {
                    // enforce turbofish exists with exactly one type argument
                    let Some(AngleBracketedGenericArguments { args: generic_args, .. }) = turbofish
                    else {
                        return Err(syn::Error::new(
                            turbofish.span(),
                            "The special deref method requires a type annotation for the expected dereferenced type",
                        ));
                    };
                    if generic_args.len() != 1 {
                        return Err(syn::Error::new(
                            generic_args.span(),
                            "The deref method requires exactly one generic argument, the type annotation for the expected dereferenced type",
                        ));
                    }
                    let generic_arg = generic_args.into_iter().next().unwrap();
                    let GenericArgument::Type(ty) = generic_arg else {
                        return Err(syn::Error::new(
                            generic_arg.span(),
                            "The generic argument for the expected dereferenced type must be a type",
                        ));
                    };

                    // enforce no arguments
                    if args.len() != 0 {
                        return Err(syn::Error::new(
                            args.span(),
                            "The deref method does not accept any arguments",
                        ));
                    }

                    Ok(Place::Deref { receiver, method, expected_type: ty })
                }
                "index" => {
                    // enforce no turbofish
                    if turbofish.is_some() {
                        return Err(syn::Error::new(
                            turbofish.as_ref().unwrap().span(),
                            "The index method does not accept any generic arguments",
                        ));
                    }

                    // enforce single argument that is a single identifier
                    if args.len() != 1 {
                        return Err(syn::Error::new(
                            args.span(),
                            "The index method requires exactly one argument, the index to use",
                        ));
                    }
                    let arg = args.into_iter().next().unwrap();
                    let index = expr_path_to_ident(arg)?;

                    Ok(Place::Index { receiver, method, index })
                }
                "cast" => {
                    // enforce turbofish exists with exactly one type argument
                    let Some(AngleBracketedGenericArguments { args: generic_args, .. }) = turbofish
                    else {
                        return Err(syn::Error::new(
                            turbofish.span(),
                            "The special dyn_field method requires a type annotation for the expected projected type",
                        ));
                    };
                    if generic_args.len() != 1 {
                        return Err(syn::Error::new(
                            generic_args.span(),
                            "The dyn_field method requires exactly one generic argument, the type annotation for the expected projected type",
                        ));
                    }
                    let generic_arg = generic_args.into_iter().next().unwrap();
                    let GenericArgument::Type(ty) = generic_arg else {
                        return Err(syn::Error::new(
                            generic_arg.span(),
                            "The generic argument for the expected projected type must be a type",
                        ));
                    };

                    // enforce no arguments
                    if args.len() != 0 {
                        return Err(syn::Error::new(
                            args.span(),
                            "The deref method does not accept any arguments",
                        ));
                    }

                    Ok(Place::Cast { receiver, method, expected_type: ty })
                }
                _ => Err(syn::Error::new(
                    method.span(),
                    "Method call does not match known method calls for place expressions",
                )),
            }
        }
        Expr::Macro(ExprMacro { attrs, mac }) => {
            no_attrs!(attrs);
            if let Some(macro_ident) = mac.path.get_ident()
                && macro_ident == "place_use"
            {
                let ident: Ident = syn::parse2(mac.tokens.clone())?;
                Ok(Place::PlaceUse(ident))
            } else {
                Err(syn::Error::new(mac.span(), "Expected a place_use expression"))
            }
        }
        _ => Err(syn::Error::new(expr.span(), "Expected a place expression")),
    }
}

fn expr_to_place_operand(expr: Expr) -> syn::Result<PlaceOperand> {
    match expr {
        Expr::Reference(ExprReference { attrs, and_token, mutability, expr: inner }) => {
            no_attrs!(attrs);

            let expr = expr_to_place(*inner)?;
            Ok(PlaceOperand { borrow: Some((and_token, mutability)), place: expr })
        }
        other => {
            let place = expr_to_place(other)?;
            Ok(PlaceOperand { borrow: None, place })
        }
    }
}

fn expr_path_to_ident(expr: Expr) -> syn::Result<Ident> {
    let Expr::Path(ExprPath { attrs, qself: None, path }) = expr else {
        return Err(syn::Error::new(expr.span(), "Expected a single identifier"));
    };

    no_attrs!(attrs);

    path.get_ident().ok_or(syn::Error::new(path.span(), "Expected a single identifier")).cloned()
}

fn expr_to_operation(expr: Expr) -> syn::Result<Operation> {
    match expr {
        Expr::Binary(ExprBinary { attrs, left, op, right }) => {
            no_attrs!(attrs);
            let lhs = expr_to_place_operand(*left)?;
            let rhs = expr_to_place_operand(*right)?;
            Ok(Operation::BinaryOp { op, lhs, rhs })
        }
        Expr::Unary(ExprUnary { attrs, op, expr }) => {
            no_attrs!(attrs);
            let operand = expr_to_place_operand(*expr)?;
            Ok(Operation::UnaryOp { op, operand })
        }
        Expr::Call(ExprCall { attrs, func, args, .. }) => {
            no_attrs!(attrs);

            match *func {
                Expr::Const(ExprConst { attrs, const_token: _, block }) => {
                    let const_block_span = block.span();
                    no_attrs!(attrs);
                    if block.stmts.len() != 1 {
                        return Err(syn::Error::new(
                            block.span(),
                            "Host function calls must have exactly one statement",
                        ));
                    }

                    let stmt = block.stmts.into_iter().next().unwrap();
                    let syn::Stmt::Expr(path, None) = stmt else {
                        return Err(syn::Error::new(
                            const_block_span,
                            "Expected a path to a host function",
                        ));
                    };

                    let Expr::Path(ExprPath { attrs, qself: None, path: function }) = path else {
                        return Err(syn::Error::new(
                            path.span(),
                            "Expected a path to a host function",
                        ));
                    };
                    no_attrs!(attrs);

                    let args = args
                        .into_iter()
                        .map(expr_to_place_operand)
                        .collect::<syn::Result<Vec<_>>>()?;
                    Ok(Operation::CallHostFunction { function, args })
                }
                _ => todo!("TODO add a variant for indirect calls"),
            }
        }
        Expr::Const(ExprConst { attrs, const_token: _, block }) => {
            no_attrs!(attrs);

            Ok(Operation::Const { value: block })
        }
        other => {
            let operand = expr_to_place_operand(other)?;
            Ok(Operation::Operand(operand))
        }
    }
}

pub fn parse_type_mapping(m: &syn::ExprMacro) -> syn::Result<()> {
    // enforce path is "type_mapping"
    assert!(m.mac.path.get_ident().unwrap() == "type_mapping");

    // enforce no attributes
    no_attrs!(m.attrs);

    // enforce no arguments
    if !m.mac.tokens.is_empty() {
        return Err(syn::Error::new(m.mac.tokens.span(), "Expected no arguments"));
    }

    Ok(())
}

pub fn parse_type_of(m: &syn::ExprMacro) -> syn::Result<Expr> {
    // enforce path is "type_of"
    assert!(m.mac.path.get_ident().unwrap() == "type_of");

    // enforce no attributes
    no_attrs!(m.attrs);

    // now we know the inside can be parsed
    let inner_expr: Expr = syn::parse2(m.mac.tokens.clone())?;
    Ok(inner_expr)
}

pub fn parse_place_ref(m: &syn::ExprMacro) -> syn::Result<Expr> {
    // enforce path is "place_ref"
    assert!(m.mac.path.get_ident().unwrap() == "place_ref");

    // enforce no attributes
    no_attrs!(m.attrs);

    // now we know the inside can be parsed
    let inner_ident: Expr = syn::parse2(m.mac.tokens.clone())?;
    Ok(inner_ident)
}
