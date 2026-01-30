// #![feature(if_let_guard, slice_as_array)]

use std::path::Path;
use std::{collections::HashMap, fs, rc::Rc};

use ast::CommandBlock;

use engine::mir::Node as _;
use engine::slotmap::Key as _;
use engine::util::reflection::Reflect;
use engine::{
    mir::{
        self, CustomVarDecl, Function, FunctionId, LocalDeclaration, LocalId, LocalStorage, MirTy,
        NlAbstractTy, NodeId, NodeKind,
        node::{self, Agentset, AskRecipient, BinaryOpcode, PatchLocRelation, UnaryOpcode},
    },
    sim::{
        patch::PatchVarDesc,
        turtle::{BreedId, TurtleVarDesc},
        value::UnpackedDynBox,
    },
    slotmap::{SecondaryMap, SlotMap},
};
use tracing::{instrument, trace};

pub extern crate serde_json;

mod ast;
mod cheats;

pub use cheats::add_cheats;

use crate::ast::Ast;

#[derive(Debug, Default)]
pub struct GlobalScope {
    constants: HashMap<&'static str, fn() -> NodeKind>,
    global_vars: HashMap<Rc<str>, usize>,
    patch_vars: HashMap<Rc<str>, PatchVarDesc>,
    turtle_vars: HashMap<Rc<str>, TurtleVarDesc>,
    /// The default turtle breed is represented by the empty string.
    turtle_breeds: HashMap<Rc<str>, BreedId>,
    functions: HashMap<Rc<str>, FunctionId>,
    // TODO(mvp) add link variables
}

#[non_exhaustive]
#[derive(Debug)]
enum NameReferent {
    Constant(fn() -> NodeKind),
    Global(usize),
    TurtleVar(TurtleVarDesc),
    PatchVar(PatchVarDesc),
    #[allow(dead_code)] // remove when turtle breeds are implemented
    TurtleBreed(BreedId),
    UserProc(FunctionId),
}

impl GlobalScope {
    fn add_builtins(&mut self, default_turtle_breed: BreedId) {
        self.constants.extend([
            (
                "RED",
                (|| NodeKind::from(node::Constant { value: UnpackedDynBox::Float(15.0) }))
                    as fn() -> NodeKind,
            ),
            ("ORANGE", || NodeKind::from(node::Constant { value: UnpackedDynBox::Float(25.0) })),
            ("GREEN", || NodeKind::from(node::Constant { value: UnpackedDynBox::Float(55.0) })),
            ("CYAN", || NodeKind::from(node::Constant { value: UnpackedDynBox::Float(85.0) })),
            ("SKY", || NodeKind::from(node::Constant { value: UnpackedDynBox::Float(95.0) })),
            ("BLUE", || NodeKind::from(node::Constant { value: UnpackedDynBox::Float(105.0) })),
            ("VIOLET", || NodeKind::from(node::Constant { value: UnpackedDynBox::Float(115.0) })),
        ]);
        self.patch_vars.extend([(Rc::from("PCOLOR"), PatchVarDesc::Pcolor)]);
        self.turtle_vars.extend([
            (Rc::from("WHO"), TurtleVarDesc::Who),
            (Rc::from("COLOR"), TurtleVarDesc::Color),
            (Rc::from("SIZE"), TurtleVarDesc::Size),
        ]);
        self.turtle_breeds.extend([("".into(), default_turtle_breed)]);
    }

    fn lookup(&self, name: &str) -> Option<NameReferent> {
        let Self {
            constants,
            global_vars: globals,
            patch_vars,
            turtle_vars,
            turtle_breeds,
            functions,
        } = self;
        if let Some(mk_node) = constants.get(name) {
            return Some(NameReferent::Constant(*mk_node));
        }
        if let Some(global_id) = globals.get(name) {
            return Some(NameReferent::Global(*global_id));
        }
        if let Some(turtle_var_id) = turtle_vars.get(name) {
            return Some(NameReferent::TurtleVar(*turtle_var_id));
        }
        if let Some(patch_var_id) = patch_vars.get(name) {
            return Some(NameReferent::PatchVar(*patch_var_id));
        }
        if let Some(turtle_breed_id) = turtle_breeds.get(name) {
            return Some(NameReferent::TurtleBreed(*turtle_breed_id));
        }
        if let Some(function_id) = functions.get(name) {
            return Some(NameReferent::UserProc(*function_id));
        }
        None
    }
}

#[derive(Debug, Default)]
struct MirBuilder {
    global_names: GlobalScope,
    global_vars: Vec<CustomVarDecl>,
    turtle_breeds: SlotMap<BreedId, ()>,
    turtle_vars: Vec<CustomVarDecl>,
    patch_vars: Vec<CustomVarDecl>,
    /// Maps a function id to the function data
    functions: SecondaryMap<FunctionId, Function>,
    aux_fn_info: SlotMap<FunctionId, FnInfo>,
    locals: SlotMap<LocalId, LocalDeclaration>,
    nodes: SlotMap<NodeId, NodeKind>,
}

pub struct ParseResult {
    pub program: mir::Program,
    pub global_names: GlobalScope,
    pub fn_info: SecondaryMap<FunctionId, FnInfo>,
}

pub fn ast_to_mir(ast: Ast) -> anyhow::Result<ParseResult> {
    trace!("starting AST to MIR conversion");

    // TODO add builtin names to the global scope
    let mut mir = MirBuilder::default();

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
    } = ast;

    // create the default turtle breed
    let default_turtle_breed = mir.turtle_breeds.insert(());

    // add builtin names to the global scope
    mir.global_names.add_builtins(default_turtle_breed);

    // create the global variables
    for name in global_var_names {
        let name: Rc<str> = name.to_uppercase().into();
        let decl = CustomVarDecl { name: name.clone(), ty: NlAbstractTy::Top.into() };
        let index = mir.global_vars.len();
        mir.global_vars.push(decl);
        trace!("Added global variable `{}` at index {:?}", name, index);
        mir.global_names.global_vars.insert(name, index);
    }
    // add custom patch variables
    for (i, name) in patch_var_names.into_iter().enumerate() {
        let name: Rc<str> = name.to_uppercase().into();
        let patch_var_id = PatchVarDesc::Custom(i);
        trace!("Adding patch variable `{}` with id {:?}", name, patch_var_id);
        mir.global_names.patch_vars.insert(name.clone(), patch_var_id);
        mir.patch_vars.push(CustomVarDecl { name, ty: NlAbstractTy::Top.into() });
    }
    // add custom turtle variables
    for (i, name) in turtle_var_names.into_iter().enumerate() {
        let name: Rc<str> = name.to_uppercase().into();
        let turtle_var_id = TurtleVarDesc::Custom(i);
        trace!("Adding turtle variable `{}` with id {:?}", name, turtle_var_id);
        mir.global_names.turtle_vars.insert(name.clone(), turtle_var_id);
        mir.turtle_vars.push(CustomVarDecl { name, ty: NlAbstractTy::Top.into() });
    }

    // go through each procedure and build the function info before building the
    // bodies
    let mut bodies_to_build = Vec::new();
    for procedure_ast in procedures {
        // compile for a set of hardcoded agent classes
        use ast::AgentClass as Ac;
        let agent_class = match procedure_ast.agent_class {
            // if any agent can execute it, it's probably the observer that executes it
            Ac { observer: true, turtle: true, patch: true, link: true } => AgentClass::Observer,
            Ac { observer: true, turtle: false, patch: false, link: false } => AgentClass::Observer,
            // -TP- means it uses patch variables, which is probably for patches
            Ac { observer: false, turtle: true, patch: true, link: false } => AgentClass::Patch,
            Ac { observer: false, turtle: true, patch: false, link: false } => AgentClass::Turtle,
            // TODO(mvp) I believe the correct way to handle this is, instead of
            // just picking the most plausible agent class to generate the
            // function for, is to generate a different version of the function
            // for each agent class (except for observer functions, which any
            // agent can execute since it doesn't have a self parameter). Not
            // all of those variants will actually be needed, but we can just
            // prune the unused ones later (perhaps selectively compiling only
            // the bodies that are needed).
            _ => todo!("handle all combinations of agent classes"),
        };

        let (fn_id, body) = build_function_info(procedure_ast, agent_class, &mut mir);

        mir.global_names.functions.insert(mir.aux_fn_info[fn_id].debug_name.clone(), fn_id);

        // save the ast to build the body later
        bodies_to_build.push((fn_id, body));
    }

    // then go through each procedure and build it
    for (fn_id, body) in bodies_to_build {
        build_function_body(fn_id, body.statements, &mut mir);
    }

    Ok(ParseResult {
        program: mir::Program {
            globals: mir.global_vars.into(),
            globals_schema: None,
            turtle_breeds: mir.turtle_breeds,
            custom_turtle_vars: mir.turtle_vars,
            custom_patch_vars: mir.patch_vars,
            turtle_schema: None,
            patch_schema: None,
            functions: mir.functions.into_iter().collect(),
            locals: mir.locals,
            nodes: mir.nodes,
        },
        global_names: mir.global_names,
        fn_info: mir.aux_fn_info.into_iter().collect(),
    })
}

// The first step to building a function. This does everything except for
// building the body.
fn build_function_info(
    procedure_ast: ast::Procedure,
    agent_class: AgentClass,
    mir: &mut MirBuilder,
) -> (FunctionId, ast::CommandBlock) {
    let ast::Procedure { name, arg_names, return_type, agent_class: supported_classes, body } =
        procedure_ast;

    // verify that the procedure can support the given agent class
    match agent_class {
        AgentClass::Observer => assert!(supported_classes.observer),
        AgentClass::Turtle => assert!(supported_classes.turtle),
        AgentClass::Patch => assert!(supported_classes.patch),
        AgentClass::Link => assert!(supported_classes.link),
        AgentClass::Any => {}
    }

    // calculate the function parameters
    let mut positional_params = Vec::new();
    // always add the context parameter
    let context_param = mir.locals.insert(LocalDeclaration {
        debug_name: Some("context".into()),
        ty: (<*mut u8 as Reflect>::CONCRETE_TY).into(),
        storage: LocalStorage::Register,
    });
    positional_params.push(context_param);
    trace!("Added context parameter with local_id: {:?}", context_param);
    // add the self parameter
    let self_param = match agent_class {
        AgentClass::Observer => {
            trace!("No self parameter needed for Observer agent class");
            None
        }
        AgentClass::Turtle => {
            let local_id = mir.locals.insert(LocalDeclaration {
                debug_name: Some("self_turtle_id".into()),
                ty: NlAbstractTy::Turtle.into(),
                storage: LocalStorage::Register,
            });
            trace!("Added turtle self parameter with local_id: {:?}", local_id);
            Some(local_id)
        }
        AgentClass::Patch => {
            let local_id = mir.locals.insert(LocalDeclaration {
                debug_name: Some("self_patch_id".into()),
                ty: NlAbstractTy::Patch.into(),
                storage: LocalStorage::Register,
            });
            trace!("Added patch self parameter with local_id: {:?}", local_id);
            Some(local_id)
        }
        AgentClass::Link => todo!("TODO(mvp) add self parameter for Link agent class"),
        AgentClass::Any => todo!("TODO(mvp) add self parameter that can be any agent"),
    };
    positional_params.extend(self_param);
    // add user-defined parameters
    let mut local_names = HashMap::new();
    for name in arg_names {
        let name: Rc<str> = name.to_uppercase().into();
        let local_id = mir.locals.insert(LocalDeclaration {
            debug_name: Some(name.clone()),
            ty: NlAbstractTy::Top.into(),
            storage: LocalStorage::Register,
        });
        trace!("Adding positional parameter {} with local_id: {:?}", name, local_id);
        positional_params.push(local_id);
        local_names.insert(name, local_id);
    }

    // calculate the return type
    let return_ty = match return_type {
        ast::ReturnType::Unit => NlAbstractTy::Unit,
        ast::ReturnType::Wildcard => NlAbstractTy::Top,
    };
    trace!("calculated return type: {:?}", return_ty);

    let fn_info = FnInfo {
        debug_name: name.to_uppercase().into(),
        env_param: None,
        context_param: Some(context_param),
        self_param: self_param,
        positional_params,
        return_ty,
        local_names,
        num_internal_bodies: 0,
    };
    (mir.aux_fn_info.insert(fn_info), body)
}

fn build_function_body(fn_id: FunctionId, statements: Vec<ast::Node>, mir: &mut MirBuilder) {
    trace!("building body");

    let mut break_nodes = Vec::new();
    let mut locals = mir.aux_fn_info[fn_id].positional_params.clone();
    let body_node = translate_statement_block(
        statements,
        FnBodyBuilderCtx { mir, fn_id, locals: &mut locals, come_from: &mut break_nodes },
    );
    // point the body node and the nodes that break out of that body to each other
    for &break_node in &break_nodes {
        let NodeKind::Break(node::Break { target, value: _ }) = &mut mir.nodes[break_node] else {
            panic!("expected a break node, got {:?}", mir.nodes[break_node]);
        };

        *target = body_node;
    }
    let NodeKind::Block(node::Block { statements: _, come_from }) = &mut mir.nodes[body_node]
    else {
        panic!("expected a block node, got {:?}", mir.nodes[body_node]);
    };
    come_from.extend(break_nodes);

    let fn_info = &mir.aux_fn_info[fn_id];
    mir.functions.insert(
        fn_id,
        Function {
            debug_name: Some(fn_info.debug_name.clone()),
            parameters: fn_info.positional_params.clone(),
            return_ty: fn_info.return_ty.clone().into(),
            locals,
            root_node: body_node,
        },
    );
}

#[derive(Debug, Copy, Clone)]
enum AgentClass {
    Observer,
    Turtle,
    Patch,
    #[allow(dead_code)] // remove when link procedures are implemented
    Link,
    Any,
}

/// Holds information about a function while it is being built.
#[derive(Debug)]
pub struct FnInfo {
    debug_name: Rc<str>,
    env_param: Option<LocalId>,
    context_param: Option<LocalId>,
    self_param: Option<LocalId>,
    positional_params: Vec<LocalId>,
    return_ty: NlAbstractTy,
    local_names: HashMap<Rc<str>, LocalId>,
    num_internal_bodies: usize,
}

struct FnBodyBuilderCtx<'a> {
    mir: &'a mut MirBuilder,
    fn_id: FunctionId,
    /// Tracks the local variables created in the body of the function.
    locals: &'a mut Vec<LocalId>,
    /// Lists all nodes that want to break out of this function's body. Since
    /// these nodes are constructed before the enclosing body's node, they must
    /// be modified after the fact to point to the enclosing body's node.
    come_from: &'a mut Vec<NodeId>,
}

impl<'a> FnBodyBuilderCtx<'a> {
    fn reborrow<'s>(&'s mut self) -> FnBodyBuilderCtx<'s> {
        FnBodyBuilderCtx {
            mir: self.mir,
            fn_id: self.fn_id,
            locals: self.locals,
            come_from: self.come_from,
        }
    }

    /// Returns a node that gets the context parameter for the current function
    fn get_context(&mut self) -> NodeId {
        let id = self.mir.nodes.insert(NodeKind::from(node::GetLocalVar {
            local_id: self.mir.aux_fn_info[self.fn_id]
                .context_param
                .expect("expected context parameter"),
        }));
        trace!("Got context parameter with id: {:?}", id);
        id
    }

    /// Returns a node that gets the self parameter.
    fn get_self_agent(&mut self) -> NodeId {
        let self_param =
            self.mir.aux_fn_info[self.fn_id].self_param.expect("expected self parameter");
        self.mir.nodes.insert(NodeKind::from(node::GetLocalVar { local_id: self_param }))
    }
}

fn translate_recipients(
    recips: ast::Node,
    mut ctx: FnBodyBuilderCtx<'_>,
) -> (AskRecipient, AgentClass) {
    let recipients_id = translate_node(recips, ctx.reborrow()).0;
    // TODO(mvp): Does not currently give type info on individual agents, nor breeds, nor links,
    // nor recipients within variables
    match &ctx.mir.nodes[recipients_id] {
        NodeKind::Agentset(Agentset::AllPatches) => (AskRecipient::AllPatches, AgentClass::Patch),
        NodeKind::Agentset(Agentset::AllTurtles) => (AskRecipient::AllTurtles, AgentClass::Turtle),
        _ => (AskRecipient::Any(recipients_id), AgentClass::Any),
    }
}

// Returns the id of the created MIR node as well as whether it diverges (e.g.
// early return)
#[instrument(skip_all, fields(node_type, name))]
fn translate_node(ast_node: ast::Node, mut ctx: FnBodyBuilderCtx<'_>) -> (NodeId, bool) {
    use ast::CommandCall as C;
    use ast::Node as N;
    use ast::ReporterCall as R;
    let mut breaking_node = false;
    let node = match ast_node {
        N::LetBinding { var_name, value } => {
            translate_let_binding(Rc::from(var_name.as_str()), *value, ctx.reborrow())
        }
        N::CommandProcCall { name, args } => {
            let referent = ctx
                .mir
                .global_names
                .lookup(&name)
                .unwrap_or_else(|| panic!("unknown command {:?}", name));
            let NameReferent::UserProc(target) = referent else {
                panic!("expected a user procedure, got {:?}", referent);
            };
            let mut arg_nodes = Vec::new();
            let fn_info = &ctx.mir.aux_fn_info[target];
            assert!(fn_info.env_param.is_none());
            if let Some(_) = fn_info.context_param {
                arg_nodes.push(ctx.get_context());
            }
            if let Some(_) = ctx.mir.aux_fn_info[target].self_param {
                arg_nodes.push(ctx.get_self_agent());
            }
            arg_nodes.extend(args.into_iter().map(|arg| translate_node(arg, ctx.reborrow()).0));
            trace!(
                "expected {} positional parameters",
                ctx.mir.aux_fn_info[target].positional_params.len()
            );
            trace!("added {} positional parameters", arg_nodes.len());
            assert!(ctx.mir.aux_fn_info[target].positional_params.len() == arg_nodes.len());
            NodeKind::from(node::CallUserFn { target, args: arg_nodes })
        }
        N::CommandCall(C::Report([value])) => {
            breaking_node = true; // mark this node to have its target fixed later
            let value = translate_node(*value, ctx.reborrow()).0;
            NodeKind::from(node::Break { target: NodeId::null(), value: Some(value) })
        }
        N::CommandCall(C::Stop([])) => {
            breaking_node = true; // mark this node to have its target fixed later
            NodeKind::from(node::Break { target: NodeId::null(), value: None })
        }
        N::CommandCall(C::ClearAll([])) => {
            let context = ctx.get_context();
            NodeKind::from(node::ClearAll { context })
        }
        N::CommandCall(C::CreateTurtles([population, body])) => {
            let context = ctx.get_context();
            let population = translate_node(*population, ctx.reborrow()).0;
            let body =
                translate_ephemeral_closure(*body, ctx.fn_id, AgentClass::Turtle, ctx.reborrow());
            NodeKind::from(node::CreateTurtles {
                context,
                breed: ctx.mir.global_names.turtle_breeds[""], // TODO(mvp) add creating other turtle breeds
                num_turtles: population,
                body,
            })
        }
        N::CommandCall(C::Set([var, value])) => {
            let var_name = translate_var_reporter_without_read(var.as_ref());
            let var_desc = ctx.mir.global_names.lookup(var_name).unwrap();
            let value = translate_node(*value, ctx.reborrow()).0;

            // the kind of variable being  assigned determines which node to use
            match var_desc {
                NameReferent::TurtleVar(var) => {
                    let context = ctx.get_context();
                    let turtle = ctx.get_self_agent();
                    NodeKind::from(node::SetTurtleVar { context, turtle, var, value })
                }
                NameReferent::PatchVar(var) => {
                    let context = ctx.get_context();
                    let agent = ctx.get_self_agent();
                    NodeKind::from(node::SetPatchVarAsTurtleOrPatch { context, agent, var, value })
                }
                NameReferent::Global(_) => todo!("TODO(mvp) add setting global variables"),
                other => panic!("cannot mutate value of {:?}", other),
            }
        }
        N::CommandCall(C::Fd([distance])) => {
            let context = ctx.get_context();
            let turtle = ctx.get_self_agent();
            let distance = translate_node(*distance, ctx.reborrow()).0;
            NodeKind::from(node::TurtleForward { context, turtle, distance })
        }
        N::CommandCall(C::Left([heading])) => {
            let context = ctx.get_context();
            let turtle = ctx.get_self_agent();
            let angle_rt = translate_node(*heading, ctx.reborrow()).0;
            let angle_lt = ctx
                .mir
                .nodes
                .insert(NodeKind::from(node::UnaryOp { op: UnaryOpcode::Neg, operand: angle_rt }));
            NodeKind::from(node::TurtleRotate { context, turtle, angle: angle_lt })
        }
        N::CommandCall(C::Right([heading])) => {
            let context = ctx.get_context();
            let turtle = ctx.get_self_agent();
            let angle = translate_node(*heading, ctx.reborrow()).0;
            NodeKind::from(node::TurtleRotate { context, turtle, angle })
        }
        N::CommandCall(C::ResetTicks([])) => {
            NodeKind::from(node::ResetTicks { context: ctx.get_context() })
        }
        N::CommandCall(C::Ask([rs, body])) => {
            let context = ctx.get_context();
            let (recipients, agent_class) = translate_recipients(*rs, ctx.reborrow());
            let body = translate_ephemeral_closure(*body, ctx.fn_id, agent_class, ctx.reborrow());
            NodeKind::from(node::Ask { context, recipients, body })
        }
        N::CommandCall(C::If([condition, then_block])) => {
            let condition = translate_node(*condition, ctx.reborrow()).0;
            let ast::Node::CommandBlock(ast::CommandBlock { statements }) = *then_block else {
                panic!("expected a command block, got {:?}", then_block);
            };
            let then_block = translate_statement_block(statements, ctx.reborrow());
            let else_block = translate_statement_block(vec![], ctx.reborrow());
            NodeKind::from(node::IfElse { condition, then_block, else_block })
        }
        N::CommandCall(C::IfElse([condition, then_block, else_block])) => {
            let condition = translate_node(*condition, ctx.reborrow()).0;
            let ast::Node::CommandBlock(ast::CommandBlock { statements }) = *then_block else {
                panic!("expected a command block, got {:?}", then_block);
            };
            let then_block = translate_statement_block(statements, ctx.reborrow());
            let ast::Node::CommandBlock(ast::CommandBlock { statements }) = *else_block else {
                panic!("expected a command block, got {:?}", else_block);
            };
            let else_block = translate_statement_block(statements, ctx.reborrow());
            NodeKind::from(node::IfElse { condition, then_block, else_block })
        }
        N::CommandCall(C::Diffuse([variable, amt])) => {
            let var_name = translate_var_reporter_without_read(variable.as_ref());
            let Some(NameReferent::PatchVar(var_desc)) = ctx.mir.global_names.lookup(var_name)
            else {
                panic!("expected patch variable for DIFFUSE");
            };
            let context = ctx.get_context();
            let amt = translate_node(*amt, ctx.reborrow()).0;
            NodeKind::from(node::Diffuse { context, variable: var_desc, amt })
        }
        N::CommandCall(C::Tick([])) => {
            NodeKind::from(node::AdvanceTick { context: ctx.get_context() })
        }
        N::CommandCall(C::SetDefaultShape([breed, shape])) => {
            let breed = translate_node(*breed, ctx.reborrow()).0;
            let shape = translate_node(*shape, ctx.reborrow()).0;
            NodeKind::from(node::SetDefaultShape { breed, shape })
        }
        N::LetRef { name } | N::ProcedureArgRef { name } => {
            let Some(&local_id) = ctx.mir.aux_fn_info[ctx.fn_id].local_names.get(name.as_str())
            else {
                unreachable!("unknown variable reference: {}", name);
            };
            NodeKind::from(node::GetLocalVar { local_id })
        }
        N::Number { value } => {
            NodeKind::from(node::Constant { value: UnpackedDynBox::Float(value.as_f64().unwrap()) })
        }
        N::String { value: _ } => {
            // TODO(mvp_ants) implement string literals
            NodeKind::from(node::Constant { value: UnpackedDynBox::Float(0.0) })
        }
        N::List { items } => {
            let items =
                items.into_iter().map(|item| translate_node(item, ctx.reborrow()).0).collect();
            NodeKind::from(node::ListLiteral { items })
        }
        N::Nobody => NodeKind::from(node::Constant { value: UnpackedDynBox::Nobody }),
        N::ReporterProcCall { name, args } => {
            let referent = ctx.mir.global_names.lookup(&name).unwrap_or_else(|| {
                panic!("unknown reporter procedure {:?}", name);
            });
            let NameReferent::UserProc(target) = referent else {
                panic!("expected a user reporter procedure, got {:?}", referent);
            };
            let mut arg_nodes = Vec::new();
            let fn_info = &ctx.mir.aux_fn_info[target];
            assert!(fn_info.env_param.is_none());
            if let Some(_) = fn_info.context_param {
                arg_nodes.push(ctx.get_context());
            }
            if let Some(_) = ctx.mir.aux_fn_info[target].self_param {
                arg_nodes.push(ctx.get_self_agent());
            }
            arg_nodes.extend(args.into_iter().map(|arg| translate_node(arg, ctx.reborrow()).0));
            assert!(ctx.mir.aux_fn_info[target].positional_params.len() == arg_nodes.len());
            NodeKind::from(node::CallUserFn { target, args: arg_nodes })
        }
        N::GlobalVar { name } => match ctx.mir.global_names.lookup(&name) {
            Some(NameReferent::Global(index)) => {
                let context = ctx.get_context();
                NodeKind::from(node::GetGlobalVar { context, index })
            }
            Some(NameReferent::Constant(mk_node)) => mk_node(),
            _ => panic!("unknown global variable access `{}`", name),
        },
        N::TurtleVar { name } => {
            let Some(NameReferent::TurtleVar(var)) = ctx.mir.global_names.lookup(&name) else {
                panic!("unknown turtle variable access `{}`", name);
            };
            let context = ctx.get_context();
            let turtle = ctx.get_self_agent();
            NodeKind::from(node::GetTurtleVar { context, turtle, var })
        }
        N::PatchVar { name } => {
            let Some(NameReferent::PatchVar(var)) = ctx.mir.global_names.lookup(&name) else {
                panic!("unknown patch variable access `{}`", name);
            };
            let context = ctx.get_context();
            let agent = ctx.get_self_agent();
            NodeKind::from(node::GetPatchVarAsTurtleOrPatch { context, agent, var })
        }
        N::LinkVar { .. } => {
            todo!("TODO(mvp) add accessing link variables")
        }
        N::TurtleOrLinkVar { name } => {
            let Some(NameReferent::TurtleVar(var)) = ctx.mir.global_names.lookup(&name) else {
                todo!("TODO(mvp) add accessing link variables")
            };
            let context = ctx.get_context();
            let turtle = ctx.get_self_agent();
            NodeKind::from(node::GetTurtleVar { context, turtle, var })
        }
        N::ReporterCall(R::Of([body, rs])) => {
            let context = ctx.get_context();
            let (recipients, agent_class) = translate_recipients(*rs, ctx.reborrow());
            let body = translate_ephemeral_closure(*body, ctx.fn_id, agent_class, ctx.reborrow());
            NodeKind::from(node::Of { context, recipients, body })
        }
        #[rustfmt::skip]
        N::ReporterCall(reporter @ (
            | R::Lt(..)
            | R::Gt(..)
            | R::Eq(..)
            | R::Lte(..)
            | R::Gte(..)
            | R::Sub(..)
            | R::Add(..)
            | R::Mul(..)
            | R::Div(..)
            | R::And(..)
            | R::Or(..)
        )) => {
            let (op, lhs, rhs) = match reporter {
                R::Lt([lhs, rhs]) => (BinaryOpcode::Lt, lhs, rhs),
                R::Gt([lhs, rhs]) => (BinaryOpcode::Gt, lhs, rhs),
                R::Eq([lhs, rhs]) => (BinaryOpcode::Eq, lhs, rhs),
                R::Lte([lhs, rhs]) => (BinaryOpcode::Lte, lhs, rhs),
                R::Gte([lhs, rhs]) => (BinaryOpcode::Gte, lhs, rhs),
                R::Sub([lhs, rhs]) => (BinaryOpcode::Sub, lhs, rhs),
                R::Add([lhs, rhs]) => (BinaryOpcode::Add, lhs, rhs),
                R::Mul([lhs, rhs]) => (BinaryOpcode::Mul, lhs, rhs),
                R::Div([lhs, rhs]) => (BinaryOpcode::Div, lhs, rhs),
                R::And([lhs, rhs]) => (BinaryOpcode::And, lhs, rhs),
                R::Or([lhs, rhs]) => (BinaryOpcode::Or, lhs, rhs),
                _ => unreachable!(),
            };
            let context = ctx.get_context();
            let lhs = translate_node(*lhs, ctx.reborrow()).0;
            let rhs = translate_node(*rhs, ctx.reborrow()).0;
            NodeKind::from(node::BinaryOperation { context, op, lhs, rhs })
        }
        N::ReporterCall(R::Not([operand])) => {
            let operand = translate_node(*operand, ctx.reborrow()).0;
            NodeKind::from(node::UnaryOp { op: UnaryOpcode::Not, operand })
        }
        N::ReporterCall(R::Distancexy([x, y])) => {
            let context = ctx.get_context();
            let agent = ctx.get_self_agent();
            let x = translate_node(*x, ctx.reborrow()).0;
            let y = translate_node(*y, ctx.reborrow()).0;
            NodeKind::from(node::Distancexy { context, agent, x, y })
        }
        N::ReporterCall(R::CanMove([distance])) => {
            let context = ctx.get_context();
            let turtle = ctx.get_self_agent();
            let distance = translate_node(*distance, ctx.reborrow()).0;
            NodeKind::from(node::CanMove { context, turtle, distance })
        }
        N::ReporterCall(
            reporter @ (R::PatchRightAndAhead { .. } | R::PatchLeftAndAhead { .. }),
        ) => {
            let (is_left, heading, distance) = match reporter {
                R::PatchRightAndAhead([heading, distance]) => (false, heading, distance),
                R::PatchLeftAndAhead([heading, distance]) => (true, heading, distance),
                _ => unreachable!(),
            };
            let context = ctx.get_context();
            let turtle = ctx.get_self_agent();
            let heading = translate_node(*heading, ctx.reborrow()).0;
            let distance = translate_node(*distance, ctx.reborrow()).0;
            let relative_loc = if is_left {
                PatchLocRelation::LeftAhead(heading)
            } else {
                PatchLocRelation::RightAhead(heading)
            };
            NodeKind::from(node::PatchRelative { context, turtle, relative_loc, distance })
        }
        N::ReporterCall(R::MaxPxcor([])) => {
            NodeKind::from(node::MaxPxcor { context: ctx.get_context() })
        }
        N::ReporterCall(R::MaxPycor([])) => {
            NodeKind::from(node::MaxPycor { context: ctx.get_context() })
        }
        N::ReporterCall(R::OneOf([xs])) => {
            let context = ctx.get_context();
            let operand = translate_node(*xs, ctx.reborrow()).0;
            NodeKind::from(node::OneOf { context, operand })
        }
        N::ReporterCall(R::ScaleColor([color, number, range1, range2])) => {
            let color = translate_node(*color, ctx.reborrow()).0;
            let number = translate_node(*number, ctx.reborrow()).0;
            let range1 = translate_node(*range1, ctx.reborrow()).0;
            let range2 = translate_node(*range2, ctx.reborrow()).0;
            NodeKind::from(node::ScaleColor { color, number, range1, range2 })
        }
        N::ReporterCall(R::Ticks([])) => {
            NodeKind::from(node::GetTick { context: ctx.get_context() })
        }
        N::ReporterCall(R::Random([bound])) => {
            let bound = translate_node(*bound, ctx.reborrow()).0;
            NodeKind::from(node::RandomInt { context: ctx.get_context(), bound })
        }
        N::ReporterCall(R::Patches([])) => NodeKind::from(node::Agentset::AllPatches),
        N::ReporterCall(R::Turtles([])) => NodeKind::from(node::Agentset::AllTurtles),
        other => panic!("expected a statement or expression, got {:?}", other),
    };
    let node_id = ctx.mir.nodes.insert(node);
    if breaking_node {
        ctx.come_from.push(node_id);
    }
    (node_id, breaking_node)
}

fn translate_statement_block(
    statements_ast: Vec<ast::Node>,
    mut ctx: FnBodyBuilderCtx<'_>,
) -> NodeId {
    // translate each statement that appears in the ast
    let mut statements = Vec::new();
    let mut falls_through = true;
    for ast_node in statements_ast {
        let (node_id, diverges) = translate_node(ast_node, ctx.reborrow());
        statements.push(node_id);
        if diverges {
            falls_through = false;
        }
    }

    // add a break node to prevent control flow from "falling through" the end
    // of the block without an early return
    let mut additional_come_from = None;
    if falls_through {
        let break_node = ctx
            .mir
            .nodes
            .insert(NodeKind::from(node::Break { target: NodeId::null(), value: None }));
        statements.push(break_node);
        additional_come_from = Some(break_node);
    }

    let node = NodeKind::from(node::Block {
        statements,
        come_from: additional_come_from.into_iter().collect(),
    });
    let statement_block_node = ctx.mir.nodes.insert(node);

    if let Some(additional_come_from) = additional_come_from {
        let NodeKind::Break(node::Break { target, value: _ }) =
            &mut ctx.mir.nodes[additional_come_from]
        else {
            panic!("expected a break node, got {:?}", ctx.mir.nodes[additional_come_from]);
        };
        *target = statement_block_node;
    }

    statement_block_node
}

fn translate_let_binding(
    name: Rc<str>,
    value: ast::Node,
    mut ctx: FnBodyBuilderCtx<'_>,
) -> NodeKind {
    let local_id = ctx.mir.locals.insert(LocalDeclaration {
        debug_name: Some(name.clone()),
        ty: NlAbstractTy::Top.into(),
        storage: LocalStorage::Register,
    });
    ctx.locals.push(local_id);
    ctx.mir.aux_fn_info[ctx.fn_id].local_names.insert(name, local_id);
    let value = translate_node(value, ctx.reborrow()).0;
    NodeKind::from(node::SetLocalVar { local_id, value })
}

fn translate_var_reporter_without_read(ast_node: &ast::Node) -> &str {
    match ast_node {
        ast::Node::GlobalVar { name } => name,
        ast::Node::TurtleVar { name } => name,
        ast::Node::TurtleOrLinkVar { name } => name,
        ast::Node::PatchVar { name } => name,
        ast::Node::LinkVar { name } => name,
        _ => panic!("expected a variable reporter call, got {:?}", ast_node),
    }
}

// TODO a lot of this function should be deduplicated from
// create_procedure_skeleton
#[instrument(skip_all)]
fn translate_ephemeral_closure(
    expr: ast::Node,
    parent_fn_id: FunctionId,
    agent_class: AgentClass,
    ctx: FnBodyBuilderCtx<'_>,
) -> NodeId {
    trace!("Translating ephemeral closure");

    // the first part of this function before the body is built is analogous tok
    // build_function_info

    // generate a procedure name
    let parent_fn_info = &mut ctx.mir.aux_fn_info[parent_fn_id];
    let parent_fn_bodies = &mut parent_fn_info.num_internal_bodies;
    let proc_name = Rc::from(format!("{} body {}", parent_fn_info.debug_name, *parent_fn_bodies));
    *parent_fn_bodies += 1;

    // calculate the function parameters
    let mut positional_params = Vec::new();
    // add the environment pointer
    let env_param = ctx.mir.locals.insert(LocalDeclaration {
        debug_name: Some("env".into()),
        ty: (<*mut u8 as Reflect>::CONCRETE_TY).into(),
        storage: LocalStorage::Register,
    });
    positional_params.push(env_param);
    // add the context parameter
    let context_param = ctx.mir.locals.insert(LocalDeclaration {
        debug_name: Some("context".into()),
        ty: (<*mut u8 as Reflect>::CONCRETE_TY).into(),
        storage: LocalStorage::Register,
    });
    positional_params.push(context_param);
    // add the self parameter
    let self_param = match agent_class {
        AgentClass::Observer => None,
        AgentClass::Turtle => {
            let local_id = ctx.mir.locals.insert(LocalDeclaration {
                debug_name: Some("self_turtle_id".into()),
                ty: NlAbstractTy::Turtle.into(),
                storage: LocalStorage::Register,
            });
            Some(local_id)
        }
        AgentClass::Patch => {
            let local_id = ctx.mir.locals.insert(LocalDeclaration {
                debug_name: Some("self_patch_id".into()),
                ty: NlAbstractTy::Patch.into(),
                storage: LocalStorage::Register,
            });
            Some(local_id)
        }
        AgentClass::Link => todo!("TODO(mvp) add self parameter with link type"),
        AgentClass::Any => {
            let local_id = ctx.mir.locals.insert(LocalDeclaration {
                debug_name: Some("self_any".into()),
                ty: NlAbstractTy::Top.into(),
                storage: LocalStorage::Register,
            });
            Some(local_id)
        }
    };
    positional_params.extend(self_param);

    let fn_info = FnInfo {
        debug_name: proc_name,
        env_param: Some(env_param),
        context_param: Some(context_param),
        self_param,
        positional_params,
        // I think we should be able to leave it like this at first and have
        // type inference fix it up for us
        return_ty: NlAbstractTy::Top.into(),
        local_names: HashMap::new(),
        num_internal_bodies: 0,
    };
    let fn_id = ctx.mir.aux_fn_info.insert(fn_info);

    // build the function body
    let statements = match expr {
        ast::Node::CommandBlock(ast::CommandBlock { statements }) => statements,
        ast::Node::ReporterBlock { reporter_app } => vec![*reporter_app],
        _ => panic!("expected a command or reporter block, got {:?}", expr),
    };
    build_function_body(fn_id, statements, ctx.mir);

    // return a closure object
    ctx.mir.nodes.insert(NodeKind::from(node::Closure {
        captures: vec![], // TODO(mvp) find which variables are captured by the closure
        body: fn_id,
    }))
}

#[instrument(skip_all)]
pub fn write_dot(program: &mir::Program, fn_id: FunctionId, prefix: &str) {
    let dot_string = mir::graphviz::to_dot_string_with_options(program, fn_id, true);
    let filename = format!(
        "dots/{}-{}-{:?}.dot",
        prefix,
        fn_id,
        program.functions[fn_id].debug_name.as_deref().unwrap_or("unnamed")
    );
    trace!("Writing DOT file for function {:?}: {}", fn_id, filename);

    if let Some(parent) = Path::new(&filename).parent()
        && let Err(e) = fs::create_dir_all(parent)
    {
        panic!("Failed to create parent directory for {} | {:?}", filename, e);
    }

    fs::write(filename, dot_string).expect("Failed to write DOT file");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ants() {
        tracing_subscriber::fmt().with_max_level(tracing::Level::TRACE).init();

        let json = include_str!("../../bench/models/ants/ast.json");
        let ast: ast::Ast = serde_json::from_str(json).unwrap();

        std::fs::write("ast_debug.txt", format!("{:#?}", ast))
            .expect("Failed to write AST debug output");

        let ParseResult { program, .. } = ast_to_mir(ast).unwrap();

        let debug_output = format!("{:#?}", program);
        std::fs::write("mir_debug.txt", debug_output).expect("Failed to write MIR debug output");

        for fn_id in program.functions.keys() {
            write_dot(&program, fn_id, "debug");
        }
    }
}
