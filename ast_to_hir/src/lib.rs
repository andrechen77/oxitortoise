use std::{collections::BTreeMap, iter, sync::Arc};

use engine::{
    hir::{
        self, CustomVarDecl, ExprKind, Function, FunctionId, Label, LocalDecl, LocalId,
        NlAbstractTy,
        expr::{self, PatchLocRelation},
    },
    sim::{
        patch::PatchVarDesc,
        turtle::{TurtleBreed, TurtleBreedId, TurtleVarDesc},
        value::{BoxedAny, NlString, UnpackedAny},
    },
};
use tracing::trace;

use crate::{
    ast::Ast,
    global_scope::{DEFAULT_TURTLE_BREED_NAME, DEFAULT_TURTLE_BREED_SINGULAR_NAME, GlobalScope},
};

mod ast;
mod global_scope;

pub extern crate serde_json;

pub struct HirResult {
    pub program: hir::Program,
    pub global_names: GlobalScope,
}

#[derive(Default)]
pub struct HirBuilder {
    next_local_id: u32,
    next_function_id: u32,
    next_label: u32,
    next_turtle_breed_id: u32,
    global_names: GlobalScope,
}

impl HirBuilder {
    fn new() -> Self {
        Default::default()
    }

    fn next_local_id(&mut self) -> LocalId {
        let id = LocalId(self.next_local_id);
        self.next_local_id += 1;
        id
    }

    fn next_function_id(&mut self) -> FunctionId {
        let id = FunctionId(self.next_function_id);
        self.next_function_id += 1;
        id
    }

    fn next_label(&mut self) -> Label {
        let id = Label(self.next_label);
        self.next_label += 1;
        id
    }

    fn next_turtle_breed_id(&mut self) -> TurtleBreedId {
        let id = TurtleBreedId(self.next_turtle_breed_id);
        self.next_turtle_breed_id += 1;
        id
    }
}

pub fn ast_to_hir(ast: Ast) -> anyhow::Result<HirResult> {
    trace!("starting AST to HIR conversion");

    let Ast {
        global_names:
            ast::GlobalNames {
                global_vars: global_var_names,
                turtle_vars: turtle_var_names,
                patch_vars: patch_var_names,
                // TODO(mvp) generate link variable registry from names
                link_vars: _link_var_names,
            },
        procedures,
        widgets,
    } = ast;

    let mut hir_builder = HirBuilder::new();

    // create global variables
    let mut global_vars = Vec::new();
    for (var_idx, global_var_name) in global_var_names.into_iter().enumerate() {
        let global_var_name: Arc<str> = global_var_name.to_uppercase().into();
        let decl = CustomVarDecl { name: global_var_name.clone(), ty: NlAbstractTy::NlTop };
        global_vars.push(decl);
        hir_builder.global_names.global_vars.insert(global_var_name, var_idx);
    }

    // create custom turtle variables
    let mut custom_turtle_vars = Vec::new();
    for (var_idx, turtle_var_name) in turtle_var_names.into_iter().enumerate() {
        let turtle_var_name: Arc<str> = turtle_var_name.to_uppercase().into();
        let decl = CustomVarDecl { name: turtle_var_name.clone(), ty: NlAbstractTy::NlTop };
        custom_turtle_vars.push(decl);
        hir_builder
            .global_names
            .turtle_vars
            .insert(turtle_var_name, TurtleVarDesc::Custom(var_idx));
    }

    // create turtle breeds
    let mut turtle_breeds = BTreeMap::new();
    let default_turtle_breed = TurtleBreed {
        name: DEFAULT_TURTLE_BREED_NAME.into(),
        singular_name: DEFAULT_TURTLE_BREED_SINGULAR_NAME.into(),
        // TODO(mvp) only assign custom variables to those that are actually
        // used by the breed in question.
        custom_variables: (0..custom_turtle_vars.len()).collect(),
    };
    let default_turtle_breed_id = hir_builder.next_turtle_breed_id();
    turtle_breeds.insert(default_turtle_breed_id, default_turtle_breed);
    hir_builder
        .global_names
        .turtle_breeds
        .insert(DEFAULT_TURTLE_BREED_NAME.into(), default_turtle_breed_id);

    // create custom patch variables
    let mut custom_patch_vars = Vec::new();
    for (var_idx, patch_var_name) in patch_var_names.into_iter().enumerate() {
        let decl = CustomVarDecl { name: patch_var_name.clone(), ty: NlAbstractTy::NlTop };
        custom_patch_vars.push(decl);
        hir_builder.global_names.patch_vars.insert(patch_var_name, PatchVarDesc::Custom(var_idx));
    }

    // add builtin names
    hir_builder.global_names.add_builtins(default_turtle_breed_id);

    // functions to build can come from actual user-defined procedures or
    // from widgets
    let mut functions_to_translate = Vec::new();

    // collect widget procedures
    for widget in widgets {
        match widget {
            ast::Widget::Plot { .. } => {}   // TODO(mvp) handle this widget
            ast::Widget::Slider { .. } => {} // TODO(mvp) handle this widget
            ast::Widget::Button { index, on_click } => {
                let function_id = hir_builder.next_function_id();
                // imagine that the widget procedure was just a code tab
                // procedure that was unnamed and had no parameters
                let function_to_translate = FunctionToTranslate {
                    debug_name: format!("widget-button {index}").into(),
                    arg_names: vec![],
                    return_type: on_click.return_type,
                    agent_class: on_click.agent_class,
                    body: on_click.body,
                    is_entrypoint: true, // widget procedures are always entrypoints
                };
                functions_to_translate.push((function_id, function_to_translate));
            }
        }
    }

    // collect code-tab procedures
    for function in procedures {
        let function_id = hir_builder.next_function_id();
        hir_builder.global_names.functions.insert(function.name.clone(), function_id);
        let function_to_compile = FunctionToTranslate {
            debug_name: function.name.clone(),
            arg_names: function.arg_names.clone(),
            return_type: function.return_type,
            agent_class: function.agent_class,
            body: function.body,
            is_entrypoint: false, // code-tab procedures are never entrypoints
        };
        functions_to_translate.push((function_id, function_to_compile));
    }

    // translate each function body
    let mut functions = BTreeMap::new();
    let mut function_bodies = BTreeMap::new();
    for (function_id, function) in functions_to_translate {
        let (function, body) = translate_function(&mut hir_builder, function);
        functions.insert(function_id, function);
        function_bodies.insert(function_id, body);
    }

    let program = hir::Program {
        global_vars: global_vars.into(),
        turtle_breeds,
        custom_turtle_vars,
        custom_patch_vars,
        functions,
        function_bodies,
    };

    Ok(HirResult { program, global_names: hir_builder.global_names })
}

struct FnBodyBuilderCtx<'a> {
    hir: &'a mut HirBuilder,
    /// The label to target when `stop` or `report` is called. This refers to
    /// the topmost block expression that is still inside the same function body
    /// (without crossing closure boundaries).
    fn_body_label: Option<Label>,
    local_scope: Option<Box<LocalVarScope>>,
    workspace_param: LocalId,
    rng_param: LocalId,
    self_param: LocalId,
}

impl<'a> FnBodyBuilderCtx<'a> {
    fn expr_workspace(&self) -> Box<ExprKind> {
        Box::new(ExprKind::from(expr::GetLocalVar { local_id: self.workspace_param }))
    }

    fn expr_rng(&self) -> Box<ExprKind> {
        Box::new(ExprKind::from(expr::GetLocalVar { local_id: self.rng_param }))
    }

    fn expr_self(&self) -> Box<ExprKind> {
        Box::new(ExprKind::from(expr::GetLocalVar { local_id: self.self_param }))
    }
}

#[derive(Debug)]
struct LocalVarScope {
    decls: BTreeMap<LocalId, LocalDecl>,
    names: BTreeMap<Arc<str>, LocalId>,
    parent: Option<Box<LocalVarScope>>,
}

impl LocalVarScope {
    fn lookup_name(&self, name: &str) -> Option<LocalId> {
        if let Some(id) = self.names.get(name) {
            Some(*id)
        } else {
            self.parent.as_ref().and_then(|p| p.lookup_name(name))
        }
    }
}

struct FunctionToTranslate {
    debug_name: Arc<str>,
    arg_names: Vec<Arc<str>>,
    return_type: ast::ReturnType,
    agent_class: ast::AgentClass,
    body: ast::CommandBlock,
    is_entrypoint: bool,
}

fn translate_function(hir: &mut HirBuilder, function: FunctionToTranslate) -> (Function, ExprKind) {
    trace!("translating function {}", function.debug_name);

    let FunctionToTranslate {
        debug_name,
        arg_names,
        return_type,
        agent_class,
        body,
        is_entrypoint,
    } = function;

    let mut ctx = make_fn_ctx(hir, None, agent_class, arg_names.into_iter());
    let (body, diverges) = translate_node_with_new_scope(&mut ctx, ast::Node::CommandBlock(body));
    assert!(!diverges, "function should return");

    // the parameters are the local variables already in scope before entering
    // the function body. we could have taken this data immediately after the
    // make_fn_ctx call, but we do it after so that the translation can use
    // the data structure
    let parameters = ctx.local_scope.unwrap().decls;

    let return_ty = match return_type {
        ast::ReturnType::Unit => NlAbstractTy::Unit,
        ast::ReturnType::Wildcard => NlAbstractTy::NlTop,
    };

    // can make additional assertions based on return type and agent class here

    let function = Function { debug_name, parameters, return_ty, is_entrypoint };
    (function, body)
}

fn with_local_scope<T>(
    ctx: &mut FnBodyBuilderCtx,
    decls: BTreeMap<LocalId, LocalDecl>,
    names: BTreeMap<Arc<str>, LocalId>,
    f: impl FnOnce(&mut FnBodyBuilderCtx) -> T,
) -> (T, BTreeMap<LocalId, LocalDecl>, BTreeMap<Arc<str>, LocalId>) {
    // push a new scope
    ctx.local_scope =
        Some(Box::new(LocalVarScope { decls, names, parent: ctx.local_scope.take() }));

    trace!("entering local scope; scope: {:?}", ctx.local_scope);

    let result = f(ctx);

    trace!("exiting local scope; scope: {:?}", ctx.local_scope);

    // pop the top scope
    let LocalVarScope { decls, names, parent } = *ctx.local_scope.take().unwrap();
    ctx.local_scope = parent;

    (result, decls, names)
}

fn translate_node_with_new_scope(
    ctx: &mut FnBodyBuilderCtx,
    ast_node: ast::Node,
) -> (ExprKind, bool) {
    let ((inner_expr, diverges), decls, _) =
        with_local_scope(ctx, BTreeMap::new(), BTreeMap::new(), |ctx| {
            translate_node(ctx, ast_node)
        });
    let final_expr = if decls.is_empty() {
        inner_expr
    } else {
        ExprKind::from(expr::Scope { locals: decls, inner: Box::new(inner_expr) })
    };
    (final_expr, diverges)
}

// Returns the HIR node as well as whether it diverges (e.g. early return).
fn translate_node(ctx: &mut FnBodyBuilderCtx, ast_node: ast::Node) -> (ExprKind, bool) {
    use ast::CommandCall as C;
    use ast::Node as N;
    use ast::ReporterCall as R;

    let mut breaking_node = false;
    let expr = match ast_node {
        N::CommandBlock(block) => ExprKind::from(translate_statement_block(ctx, block)),
        N::ReporterBlock { reporter_app } => {
            return translate_node(ctx, *reporter_app);
        }
        N::LetBinding { var_name, value } => {
            // create a new local variable
            let local_id = ctx.hir.next_local_id();
            let local_decl = LocalDecl { debug_name: var_name.clone(), ty: NlAbstractTy::NlTop };
            ctx.local_scope.as_mut().unwrap().decls.insert(local_id, local_decl);
            ctx.local_scope.as_mut().unwrap().names.insert(var_name, local_id);

            // emit a set statement that defines tha local variable
            let (value, diverges) = translate_node(ctx, *value);
            breaking_node &= diverges;
            ExprKind::from(expr::SetLocalVar { local_id, value: Box::new(value) })
        }
        N::CommandProcCall { name, args } | N::ReporterProcCall { name, args } => {
            let Some(&target) = ctx.hir.global_names.functions.get(&name) else {
                panic!("unknown command {:?}", name);
            };
            let mut arg_exprs = Vec::new();
            arg_exprs.push(ctx.expr_workspace());
            arg_exprs.push(ctx.expr_rng());
            arg_exprs.push(ctx.expr_self());
            for arg in args {
                let (arg_expr, diverges) = translate_node(ctx, arg);
                breaking_node &= diverges;
                arg_exprs.push(Box::new(arg_expr));
            }
            ExprKind::from(expr::CallUserFn { target, args: arg_exprs })
        }
        N::CommandCall(C::Report([value])) => {
            breaking_node = true;
            let (value, _diverges) = translate_node(ctx, *value);
            // could emit a lint here if the evaluation of the value itself diverges
            let target = ctx.fn_body_label.expect("report call should have a block to break from");
            ExprKind::from(expr::Break { target, value: Box::new(value) })
        }
        N::CommandCall(C::Stop([])) => {
            breaking_node = true;
            let target = ctx.fn_body_label.expect("stop call should have a block to break from");
            ExprKind::from(expr::Break {
                target,
                value: Box::new(ExprKind::from(expr::Constant { value: None })),
            })
        }
        N::CommandCall(C::ClearAll([])) => {
            ExprKind::from(expr::ClearAll { workspace: ctx.expr_workspace() })
        }
        N::CommandCall(C::CreateTurtles([num_turtles, body])) => {
            let (num_turtles, diverges) = translate_node(ctx, *num_turtles);
            breaking_node &= diverges;
            let body =
                translate_ephemeral_closure(ctx, iter::empty(), ast::AgentClass::TURTLE, *body);
            let breed = ctx.hir.global_names.turtle_breeds[DEFAULT_TURTLE_BREED_NAME];
            ExprKind::from(expr::CreateTurtles {
                workspace: ctx.expr_workspace(),
                rng: ctx.expr_rng(),
                breed,
                num_turtles: Box::new(num_turtles),
                body: Box::new(body),
            })
        }
        N::CommandCall(C::Set([var, value])) => {
            let (value, diverges) = translate_node(ctx, *value);
            breaking_node &= diverges;
            match *var {
                ast::Node::GlobalVar { name } => {
                    let _var_idx = ctx.hir.global_names.global_vars[&name];
                    todo!("TODO(mvp) add setting global variables")
                }
                ast::Node::TurtleVar { name } => {
                    let var = ctx.hir.global_names.turtle_vars[&name];
                    ExprKind::from(expr::SetTurtleVar {
                        workspace: ctx.expr_workspace(),
                        turtle: ctx.expr_self(),
                        var,
                        value: Box::new(value),
                    })
                }
                ast::Node::PatchVar { name } => {
                    let var = ctx.hir.global_names.patch_vars[&name];
                    ExprKind::from(expr::SetPatchVar {
                        workspace: ctx.expr_workspace(),
                        patch: ctx.expr_self(),
                        var,
                        value: Box::new(value),
                    })
                }
                ast::Node::TurtleOrLinkVar { name } => {
                    // TODO(mvp) also handle the case where this is a link variable
                    let var = ctx.hir.global_names.turtle_vars[&name];
                    ExprKind::from(expr::SetTurtleVar {
                        workspace: ctx.expr_workspace(),
                        turtle: ctx.expr_self(),
                        var,
                        value: Box::new(value),
                    })
                }
                _ => panic!("cannot set value of {:?}", var),
            }
        }
        N::CommandCall(C::Fd([distance])) => {
            let (distance, diverges) = translate_node(ctx, *distance);
            breaking_node &= diverges;
            ExprKind::from(expr::TurtleForward {
                workspace: ctx.expr_workspace(),
                turtle: ctx.expr_self(),
                distance: Box::new(distance),
            })
        }
        N::CommandCall(C::Left([heading])) => {
            let (heading, diverges) = translate_node(ctx, *heading);
            breaking_node &= diverges;
            ExprKind::from(expr::TurtleRotate {
                workspace: ctx.expr_workspace(),
                turtle: ctx.expr_self(),
                angle: Box::new(ExprKind::from(expr::Negate { operand: Box::new(heading) })),
            })
        }
        N::CommandCall(C::Right([heading])) => {
            let (heading, diverges) = translate_node(ctx, *heading);
            breaking_node &= diverges;
            ExprKind::from(expr::TurtleRotate {
                workspace: ctx.expr_workspace(),
                turtle: ctx.expr_self(),
                angle: Box::new(heading),
            })
        }
        N::CommandCall(C::ResetTicks([])) => {
            ExprKind::from(expr::ResetTicks { workspace: ctx.expr_workspace() })
        }
        N::CommandCall(C::Ask([rs, body])) => {
            let (recipients, diverges) = translate_node(ctx, *rs);
            breaking_node &= diverges;

            let body =
                translate_ephemeral_closure(ctx, std::iter::empty(), ast::AgentClass::ANY, *body);

            ExprKind::from(expr::Ask {
                workspace: ctx.expr_workspace(),
                rng: ctx.expr_rng(),
                recipients: Box::new(recipients),
                body: Box::new(body),
            })
        }
        N::ReporterCall(R::Of([body, rs])) => {
            let (recipients, diverges) = translate_node(ctx, *rs);
            breaking_node &= diverges;

            let body =
                translate_ephemeral_closure(ctx, std::iter::empty(), ast::AgentClass::ANY, *body);

            ExprKind::from(expr::Of {
                workspace: ctx.expr_workspace(),
                rng: ctx.expr_rng(),
                recipients: Box::new(recipients),
                body: Box::new(body),
            })
        }
        N::CommandCall(C::If([condition, then_block])) => {
            let (condition, diverges) = translate_node(ctx, *condition);
            breaking_node &= diverges;
            let (then_block, diverges) = translate_node_with_new_scope(ctx, *then_block);
            breaking_node &= diverges;
            ExprKind::from(expr::IfElse {
                condition: Box::new(condition),
                then: Box::new(then_block),
                r#else: None,
            })
        }
        N::CommandCall(C::IfElse([condition, then_block, else_block])) => {
            let (condition, diverges) = translate_node(ctx, *condition);
            breaking_node &= diverges;
            let (then_block, diverges) = translate_node_with_new_scope(ctx, *then_block);
            breaking_node &= diverges;
            let (else_block, diverges) = translate_node_with_new_scope(ctx, *else_block);
            breaking_node &= diverges;
            ExprKind::from(expr::IfElse {
                condition: Box::new(condition),
                then: Box::new(then_block),
                r#else: Some(Box::new(else_block)),
            })
        }
        N::CommandCall(C::Diffuse([variable, amt])) => {
            let ast::Node::PatchVar { name } = *variable else {
                panic!("expected a patch variable, got {:?}", variable);
            };
            let variable = ctx.hir.global_names.patch_vars[&name];
            let (amt, diverges) = translate_node(ctx, *amt);
            breaking_node &= diverges;
            ExprKind::from(expr::Diffuse {
                workspace: ctx.expr_workspace(),
                variable,
                amt: Box::new(amt),
            })
        }
        N::CommandCall(C::Tick([])) => {
            ExprKind::from(expr::AdvanceTick { workspace: ctx.expr_workspace() })
        }
        N::CommandCall(C::SetDefaultShape([breed, shape])) => {
            let (shape, diverges) = translate_node(ctx, *shape);
            breaking_node &= diverges;
            match *breed {
                ast::Node::ReporterCall(R::Turtles([])) => {
                    let breed = ctx.hir.global_names.turtle_breeds[DEFAULT_TURTLE_BREED_NAME];
                    ExprKind::from(expr::SetDefaultShape {
                        workspace: ctx.expr_workspace(),
                        breed,
                        shape: Box::new(shape),
                    })
                }
                _ => panic!("unrecognized agent class/breed {:?}", breed),
            }
        }
        N::CommandCall(C::Plotxy([_x, _y])) => {
            todo!("TODO(mvp) implement plotxy HIR expr")
        }
        N::LetRef { name } | N::ProcedureArgRef { name } => {
            let local_id = ctx.local_scope.as_ref().unwrap().lookup_name(&name).unwrap();
            ExprKind::from(expr::GetLocalVar { local_id })
        }
        N::Number { value } => ExprKind::from(expr::Constant {
            value: Some(UnpackedAny::Float(value.as_f64().unwrap())),
        }),
        N::String { value } => ExprKind::from(expr::Constant {
            value: Some(UnpackedAny::Other(BoxedAny::new(NlString::new(&value)))),
        }),
        N::List { items } => {
            let items = items
                .into_iter()
                .map(|item| {
                    let (item, diverges) = translate_node(ctx, item);
                    breaking_node &= diverges;
                    Box::new(item)
                })
                .collect();
            ExprKind::from(expr::ListLiteral { items })
        }
        N::Nobody => ExprKind::from(expr::Constant { value: Some(UnpackedAny::Nobody) }),
        N::GlobalVar { name } => {
            if let Some(&var_idx) = ctx.hir.global_names.global_vars.get(&name) {
                ExprKind::from(expr::GetGlobalVar {
                    workspace: ctx.expr_workspace(),
                    index: var_idx,
                })
            } else if let Some(value) = ctx.hir.global_names.constants.get(name.as_ref()) {
                ExprKind::clone(value)
            } else {
                panic!("unknown name {:?}", name);
            }
        }
        N::TurtleVar { name } => {
            let var = ctx.hir.global_names.turtle_vars[&name];
            ExprKind::from(expr::GetTurtleVar {
                workspace: ctx.expr_workspace(),
                turtle: ctx.expr_self(),
                var,
            })
        }
        N::PatchVar { name } => {
            let var = ctx.hir.global_names.patch_vars[&name];
            ExprKind::from(expr::GetPatchVar {
                workspace: ctx.expr_workspace(),
                patch: ctx.expr_self(),
                var,
            })
        }
        N::LinkVar { name: _ } => {
            todo!("TODO(mvp) add accessing link variables")
        }
        N::TurtleOrLinkVar { name } => {
            // TODO(mvp) also handle the case where this is a link variable
            let var = ctx.hir.global_names.turtle_vars[&name];
            ExprKind::from(expr::GetTurtleVar {
                workspace: ctx.expr_workspace(),
                turtle: ctx.expr_self(),
                var,
            })
        }
        N::ReporterCall(R::Lt([lhs, rhs])) => {
            let (lhs, diverges) = translate_node(ctx, *lhs);
            breaking_node &= diverges;
            let (rhs, diverges) = translate_node(ctx, *rhs);
            breaking_node &= diverges;
            ExprKind::from(expr::BinaryCmp {
                op: expr::BinaryCmpOpcode::Lt,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            })
        }
        N::ReporterCall(R::Gt([lhs, rhs])) => {
            let (lhs, diverges) = translate_node(ctx, *lhs);
            breaking_node &= diverges;
            let (rhs, diverges) = translate_node(ctx, *rhs);
            breaking_node &= diverges;
            ExprKind::from(expr::BinaryCmp {
                op: expr::BinaryCmpOpcode::Gt,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            })
        }
        N::ReporterCall(R::Lte([lhs, rhs])) => {
            let (lhs, diverges) = translate_node(ctx, *lhs);
            breaking_node &= diverges;
            let (rhs, diverges) = translate_node(ctx, *rhs);
            breaking_node &= diverges;
            ExprKind::from(expr::BinaryCmp {
                op: expr::BinaryCmpOpcode::Lte,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            })
        }
        N::ReporterCall(R::Gte([lhs, rhs])) => {
            let (lhs, diverges) = translate_node(ctx, *lhs);
            breaking_node &= diverges;
            let (rhs, diverges) = translate_node(ctx, *rhs);
            breaking_node &= diverges;
            ExprKind::from(expr::BinaryCmp {
                op: expr::BinaryCmpOpcode::Gte,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            })
        }
        N::ReporterCall(R::Eq([lhs, rhs])) => {
            let (lhs, diverges) = translate_node(ctx, *lhs);
            breaking_node &= diverges;
            let (rhs, diverges) = translate_node(ctx, *rhs);
            breaking_node &= diverges;
            ExprKind::from(expr::BinaryCmp {
                op: expr::BinaryCmpOpcode::Eq,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            })
        }
        N::ReporterCall(R::Add([lhs, rhs])) => {
            let (lhs, diverges) = translate_node(ctx, *lhs);
            breaking_node &= diverges;
            let (rhs, diverges) = translate_node(ctx, *rhs);
            breaking_node &= diverges;
            ExprKind::from(expr::BinaryArith {
                op: expr::BinaryArithOpcode::Add,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            })
        }
        N::ReporterCall(R::Sub([lhs, rhs])) => {
            let (lhs, diverges) = translate_node(ctx, *lhs);
            breaking_node &= diverges;
            let (rhs, diverges) = translate_node(ctx, *rhs);
            breaking_node &= diverges;
            ExprKind::from(expr::BinaryArith {
                op: expr::BinaryArithOpcode::Sub,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            })
        }
        N::ReporterCall(R::Mul([lhs, rhs])) => {
            let (lhs, diverges) = translate_node(ctx, *lhs);
            breaking_node &= diverges;
            let (rhs, diverges) = translate_node(ctx, *rhs);
            breaking_node &= diverges;
            ExprKind::from(expr::BinaryArith {
                op: expr::BinaryArithOpcode::Mul,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            })
        }
        N::ReporterCall(R::Div([lhs, rhs])) => {
            let (lhs, diverges) = translate_node(ctx, *lhs);
            breaking_node &= diverges;
            let (rhs, diverges) = translate_node(ctx, *rhs);
            breaking_node &= diverges;
            ExprKind::from(expr::BinaryArith {
                op: expr::BinaryArithOpcode::Div,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            })
        }
        N::ReporterCall(R::And([lhs, rhs])) => {
            let (lhs, diverges) = translate_node(ctx, *lhs);
            breaking_node &= diverges;
            let (rhs, diverges) = translate_node(ctx, *rhs);
            breaking_node &= diverges;
            ExprKind::from(expr::BinaryBool {
                op: expr::BinaryBoolOpcode::And,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            })
        }
        N::ReporterCall(R::Or([lhs, rhs])) => {
            let (lhs, diverges) = translate_node(ctx, *lhs);
            breaking_node &= diverges;
            let (rhs, diverges) = translate_node(ctx, *rhs);
            breaking_node &= diverges;
            ExprKind::from(expr::BinaryBool {
                op: expr::BinaryBoolOpcode::Or,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            })
        }
        N::ReporterCall(R::Not([operand])) => {
            let (operand, diverges) = translate_node(ctx, *operand);
            breaking_node &= diverges;
            ExprKind::from(expr::LogicalNot { operand: Box::new(operand) })
        }
        N::ReporterCall(R::Distancexy([x, y])) => {
            let (x, diverges) = translate_node(ctx, *x);
            breaking_node &= diverges;
            let (y, diverges) = translate_node(ctx, *y);
            breaking_node &= diverges;
            ExprKind::from(expr::Distancexy {
                workspace: ctx.expr_workspace(),
                agent: ctx.expr_self(),
                x: Box::new(x),
                y: Box::new(y),
            })
        }
        N::ReporterCall(R::CanMove([distance])) => {
            let (distance, diverges) = translate_node(ctx, *distance);
            breaking_node &= diverges;
            ExprKind::from(expr::CanMove {
                workspace: ctx.expr_workspace(),
                turtle: ctx.expr_self(),
                distance: Box::new(distance),
            })
        }
        N::ReporterCall(R::PatchRightAndAhead([heading, distance])) => {
            let (heading, diverges) = translate_node(ctx, *heading);
            breaking_node &= diverges;
            let (distance, diverges) = translate_node(ctx, *distance);
            breaking_node &= diverges;
            ExprKind::from(expr::PatchRelative {
                workspace: ctx.expr_workspace(),
                turtle: ctx.expr_self(),
                relative_loc: PatchLocRelation::RightAhead(Box::new(heading)),
                distance: Box::new(distance),
            })
        }
        N::ReporterCall(R::PatchLeftAndAhead([heading, distance])) => {
            let (heading, diverges) = translate_node(ctx, *heading);
            breaking_node &= diverges;
            let (distance, diverges) = translate_node(ctx, *distance);
            breaking_node &= diverges;
            ExprKind::from(expr::PatchRelative {
                workspace: ctx.expr_workspace(),
                turtle: ctx.expr_self(),
                relative_loc: PatchLocRelation::LeftAhead(Box::new(heading)),
                distance: Box::new(distance),
            })
        }
        N::ReporterCall(R::MaxPxcor([])) => {
            ExprKind::from(expr::MaxPxcor { workspace: ctx.expr_workspace() })
        }
        N::ReporterCall(R::MaxPycor([])) => {
            ExprKind::from(expr::MaxPycor { workspace: ctx.expr_workspace() })
        }
        N::ReporterCall(R::OneOf([xs])) => {
            let (items, diverges) = translate_node(ctx, *xs);
            breaking_node &= diverges;
            ExprKind::from(expr::OneOf { rng: ctx.expr_rng(), operand: Box::new(items) })
        }
        N::ReporterCall(R::ScaleColor([color, number, range1, range2])) => {
            let (color, diverges) = translate_node(ctx, *color);
            breaking_node &= diverges;
            let (number, diverges) = translate_node(ctx, *number);
            breaking_node &= diverges;
            let (range1, diverges) = translate_node(ctx, *range1);
            breaking_node &= diverges;
            let (range2, diverges) = translate_node(ctx, *range2);
            breaking_node &= diverges;
            ExprKind::from(expr::ScaleColor {
                color: Box::new(color),
                number: Box::new(number),
                range1: Box::new(range1),
                range2: Box::new(range2),
            })
        }
        N::ReporterCall(R::Ticks([])) => {
            ExprKind::from(expr::GetTick { workspace: ctx.expr_workspace() })
        }
        N::ReporterCall(R::Random([bound])) => {
            let (bound, diverges) = translate_node(ctx, *bound);
            breaking_node &= diverges;
            ExprKind::from(expr::RandomInt { rng: ctx.expr_rng(), bound: Box::new(bound) })
        }
        N::ReporterCall(R::Turtles([])) => ExprKind::from(expr::Agentset::AllTurtles),
        N::ReporterCall(R::Patches([])) => ExprKind::from(expr::Agentset::AllPatches),
        N::ReporterCall(R::Sum([_items])) => {
            todo!("TODO(mvp) implement sum HIR expr")
        }
        N::ReporterCall(R::With([_items, _body])) => {
            todo!("TODO(mvp) implement with HIR expr")
        }
    };
    (expr, breaking_node)
}

fn translate_statement_block(ctx: &mut FnBodyBuilderCtx, block: ast::CommandBlock) -> ExprKind {
    let ast::CommandBlock { statements: statements_ast } = block;

    let label = ctx.hir.next_label();
    if ctx.fn_body_label.is_none() {
        // since this is the topmost block expression, set any early returns
        // from the function to target this block
        ctx.fn_body_label = Some(label);
    }

    // translate each statement in the block
    let mut statements = Vec::new();
    let mut falls_through = true;
    for ast_node in statements_ast {
        let (expr, diverges) = translate_node(ctx, ast_node);
        statements.push(expr);
        if diverges {
            falls_through = false;
            // can assert here that there are no more statements
        }
    }

    // add a break node to prevent control flow from "falling through" the end
    // of the block without an early return
    if falls_through {
        let break_expr = ExprKind::from(expr::Break {
            target: label,
            value: Box::new(ExprKind::from(expr::Constant { value: None })),
        });
        statements.push(break_expr);
    }

    // after exiting the block unset ourselves as the target for early returns
    if ctx.fn_body_label == Some(label) {
        ctx.fn_body_label = None;
    }

    ExprKind::from(expr::Block { label, statements })
}

/// Creates a new scope with defaul
fn make_fn_ctx<'a>(
    hir: &'a mut HirBuilder,
    parent_scope: Option<Box<LocalVarScope>>,
    agent_class: ast::AgentClass,
    arg_names: impl Iterator<Item = Arc<str>>,
) -> FnBodyBuilderCtx<'a> {
    let mut parameters = BTreeMap::new();
    let mut names = BTreeMap::new();

    // all procedures take workspace, rng, and self parameters by default
    let workspace_param = hir.next_local_id();
    parameters.insert(
        workspace_param,
        LocalDecl { debug_name: "workspace".into(), ty: NlAbstractTy::Workspace },
    );
    let rng_param = hir.next_local_id();
    parameters.insert(rng_param, LocalDecl { debug_name: "rng".into(), ty: NlAbstractTy::Rng });
    let self_param = hir.next_local_id();
    let self_param_ty = match agent_class {
        ast::AgentClass { observer: true, turtle: false, patch: false, link: false } => {
            NlAbstractTy::Unit
        }
        ast::AgentClass { observer: false, turtle: true, patch: false, link: false } => {
            NlAbstractTy::Turtle
        }
        ast::AgentClass { observer: false, turtle: false, patch: true, link: false } => {
            NlAbstractTy::Patch
        }
        ast::AgentClass { observer: false, turtle: false, patch: false, link: true } => {
            NlAbstractTy::Link
        }
        _ => NlAbstractTy::NlTop,
    };
    parameters.insert(self_param, LocalDecl { debug_name: "self".into(), ty: self_param_ty });

    // add user-defined parameters
    for arg_name in arg_names {
        let local_id = hir.next_local_id();
        let local_decl = LocalDecl { debug_name: arg_name.clone(), ty: NlAbstractTy::NlTop };
        parameters.insert(local_id, local_decl);
        names.insert(arg_name, local_id);
    }

    let local_scope = Box::new(LocalVarScope { decls: parameters, names, parent: parent_scope });

    trace!("new function context; scope: {:?}", local_scope);

    FnBodyBuilderCtx {
        hir,
        fn_body_label: None,
        local_scope: Some(local_scope),
        workspace_param,
        rng_param,
        self_param,
    }
}

fn translate_ephemeral_closure(
    outer_ctx: &mut FnBodyBuilderCtx<'_>,
    additional_arg_names: impl Iterator<Item = Arc<str>>,
    agent_class: ast::AgentClass,
    body_ast: ast::Node,
) -> ExprKind {
    let parent_scope = outer_ctx.local_scope.take(); // remember to put it back!
    let mut closure_ctx =
        make_fn_ctx(outer_ctx.hir, parent_scope, agent_class, additional_arg_names);

    let (body, diverges) = translate_node_with_new_scope(&mut closure_ctx, body_ast);
    assert!(!diverges, "closure body should return");

    // pop the top scope from the closure context
    let LocalVarScope { decls, names: _, parent } = *closure_ctx.local_scope.take().unwrap();
    // the parameters are the local variables already in scope before entering
    // the function body. we take this after translation so that the translation
    // can use the data structure
    let parameters = decls;
    // put back the parent scope in the outer function context
    outer_ctx.local_scope = parent;

    ExprKind::from(expr::Closure {
        // TODO(mvp) find which variables are captured by the closure
        captures: vec![],
        parameters,
        body: Box::new(body),
    })
}
