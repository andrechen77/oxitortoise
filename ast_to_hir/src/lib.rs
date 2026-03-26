use std::{collections::BTreeMap, sync::Arc};

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
        widgets: _,
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

    // assign function ids to names
    let mut functions_to_build = Vec::new();
    for function in procedures {
        let function_id = hir_builder.next_function_id();
        hir_builder.global_names.functions.insert(function.name.clone(), function_id);
        functions_to_build.push((function_id, function));
    }

    // translate each function body
    let mut functions = BTreeMap::new();
    let mut function_bodies = BTreeMap::new();
    for (function_id, function) in functions_to_build {
        let (function, body) = translate_function_body(&mut hir_builder, function);
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
    fn_body_label: Option<Label>,
    local_vars: &'a mut BTreeMap<LocalId, LocalDecl>,
    workspace_param: LocalId,
    rng_param: LocalId,
    self_param: LocalId,
    local_names: BTreeMap<Arc<str>, LocalId>,
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

fn translate_function_body(
    hir: &mut HirBuilder,
    function_ast: ast::Procedure,
) -> (Function, ExprKind) {
    let ast::Procedure { name, arg_names, return_type, agent_class: _, body } = function_ast;

    let mut non_param_locals = BTreeMap::new(); // local variables that are not parametrs
    let mut parameters = BTreeMap::new(); // local variables that are parameters
    let mut local_names = BTreeMap::new(); // all local variables

    // all procedures take workspace, rng, and self parameters by default
    let workspace_param = hir.next_local_id();
    parameters.insert(
        workspace_param,
        LocalDecl { debug_name: Some("workspace".into()), ty: NlAbstractTy::Workspace },
    );
    let rng_param = hir.next_local_id();
    parameters
        .insert(rng_param, LocalDecl { debug_name: Some("rng".into()), ty: NlAbstractTy::Rng });
    let self_param = hir.next_local_id();
    parameters
        .insert(self_param, LocalDecl { debug_name: Some("self".into()), ty: NlAbstractTy::NlTop });

    // add user-defined parameters
    for arg_name in arg_names {
        let local_id = hir.next_local_id();
        let local_decl = LocalDecl { debug_name: Some(arg_name.clone()), ty: NlAbstractTy::NlTop };
        parameters.insert(local_id, local_decl);
        local_names.insert(arg_name, local_id);
    }

    let mut ctx = FnBodyBuilderCtx {
        hir,
        fn_body_label: None,
        local_vars: &mut non_param_locals,
        workspace_param,
        rng_param,
        self_param,
        local_names,
    };

    let body_without_locals = translate_statement_block(&mut ctx, body.statements);
    let body = ExprKind::from(expr::Scope {
        locals: non_param_locals,
        inner: Box::new(body_without_locals),
    });

    let return_ty = match return_type {
        ast::ReturnType::Unit => NlAbstractTy::Unit,
        ast::ReturnType::Wildcard => NlAbstractTy::NlTop,
    };

    // can make additional assertions based on return type and agent class here

    let function = Function { debug_name: Some(name.clone()), parameters, return_ty };
    (function, body)
}

// Returns the HIR node as well as whether it diverges (e.g. early return).
fn translate_node(ctx: &mut FnBodyBuilderCtx<'_>, ast_node: ast::Node) -> (ExprKind, bool) {
    use ast::CommandCall as C;
    use ast::Node as N;
    use ast::ReporterCall as R;

    let mut breaking_node = false;
    let expr = match ast_node {
        N::LetBinding { var_name, value } => {
            // create a new local variable
            let local_id = ctx.hir.next_local_id();
            let local_decl =
                LocalDecl { debug_name: Some(var_name.clone()), ty: NlAbstractTy::NlTop };
            ctx.local_vars.insert(local_id, local_decl);
            ctx.local_names.insert(var_name, local_id);

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
            let target = ctx.fn_body_label.unwrap();
            ExprKind::from(expr::Break { target, value: Box::new(value) })
        }
        N::CommandCall(C::Stop([])) => {
            breaking_node = true;
            let target = ctx.fn_body_label.unwrap();
            ExprKind::from(expr::Break {
                target,
                value: Box::new(ExprKind::from(expr::Constant { value: None })),
            })
        }
        N::CommandBlock(ast::CommandBlock { statements }) => {
            panic!("unexpected command block {:?}", statements);
        }
        N::ReporterBlock { reporter_app } => {
            panic!("unexpected reporter block {:?}", reporter_app);
        }
        N::CommandCall(C::ClearAll([])) => {
            ExprKind::from(expr::ClearAll { workspace: ctx.expr_workspace() })
        }
        N::CommandCall(C::CreateTurtles([num_turtles, body])) => {
            let (num_turtles, diverges) = translate_node(ctx, *num_turtles);
            breaking_node &= diverges;
            let body = translate_ephemeral_closure(ctx, Default::default(), *body);
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

            let closure_workspace_param = (
                ctx.hir.next_local_id(),
                LocalDecl { debug_name: Some("workspace".into()), ty: NlAbstractTy::Workspace },
            );
            let closure_rng_param = (
                ctx.hir.next_local_id(),
                LocalDecl { debug_name: Some("rng".into()), ty: NlAbstractTy::Rng },
            );
            let body = translate_ephemeral_closure(
                ctx,
                BTreeMap::from([closure_workspace_param, closure_rng_param]),
                *body,
            );

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

            let closure_workspace_param = (
                ctx.hir.next_local_id(),
                LocalDecl { debug_name: Some("workspace".into()), ty: NlAbstractTy::Workspace },
            );
            let closure_rng_param = (
                ctx.hir.next_local_id(),
                LocalDecl { debug_name: Some("rng".into()), ty: NlAbstractTy::Rng },
            );
            let body = translate_ephemeral_closure(
                ctx,
                BTreeMap::from([closure_workspace_param, closure_rng_param]),
                *body,
            );

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
            let ast::Node::CommandBlock(ast::CommandBlock { statements }) = *then_block else {
                panic!("expected a command block, got {:?}", then_block);
            };
            let then_block = translate_statement_block(ctx, statements);
            let else_block = translate_statement_block(ctx, vec![]);
            ExprKind::from(expr::IfElse {
                condition: Box::new(condition),
                then: Box::new(then_block),
                r#else: Box::new(else_block),
            })
        }
        N::CommandCall(C::IfElse([condition, then_block, else_block])) => {
            let (condition, diverges) = translate_node(ctx, *condition);
            breaking_node &= diverges;
            let ast::Node::CommandBlock(ast::CommandBlock { statements }) = *then_block else {
                panic!("expected a command block, got {:?}", then_block);
            };
            let then_block = translate_statement_block(ctx, statements);
            let ast::Node::CommandBlock(ast::CommandBlock { statements }) = *else_block else {
                panic!("expected a command block, got {:?}", else_block);
            };
            let else_block = translate_statement_block(ctx, statements);
            ExprKind::from(expr::IfElse {
                condition: Box::new(condition),
                then: Box::new(then_block),
                r#else: Box::new(else_block),
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
            let local_id = ctx.local_names[&name];
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

fn translate_statement_block(
    ctx: &mut FnBodyBuilderCtx<'_>,
    statements_ast: Vec<ast::Node>,
) -> ExprKind {
    let label = ctx.hir.next_label();
    let old_label = ctx.fn_body_label.replace(label);

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

    ctx.fn_body_label = old_label;

    ExprKind::from(expr::Block { label, statements })
}

fn translate_ephemeral_closure(
    ctx: &mut FnBodyBuilderCtx<'_>,
    parameters: BTreeMap<LocalId, LocalDecl>,
    body_ast: ast::Node,
) -> ExprKind {
    let statements = match body_ast {
        ast::Node::CommandBlock(ast::CommandBlock { statements }) => statements,
        ast::Node::ReporterBlock { reporter_app } => {
            vec![ast::Node::CommandCall(ast::CommandCall::Report([reporter_app]))]
        }
        _ => panic!("expected a command or reporter block, got {:?}", body_ast),
    };
    let body = translate_statement_block(ctx, statements);

    ExprKind::from(expr::Closure {
        // TODO(mvp) find which variables are captured by the closure
        captures: vec![],
        parameters,
        body: Box::new(body),
    })
}
