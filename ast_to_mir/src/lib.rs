// #![feature(if_let_guard, slice_as_array)]

use std::path::Path;
use std::{collections::HashMap, fs, rc::Rc};

use engine::{
    mir::{
        self, CustomVarDecl, EffectfulNode, Function, FunctionId, GlobalId, LocalDeclaration,
        LocalId, LocalStorage, MirType, NetlogoAbstractType, NodeId, StatementBlock, StatementKind,
        node::{self, BinaryOpcode, PatchLocRelation, UnaryOpcode},
    },
    sim::{
        agent_schema::{PatchSchema, TurtleSchema},
        patch::PatchVarDesc,
        turtle::{BreedId, TurtleVarDesc},
        value::{NetlogoMachineType, UnpackedDynBox},
    },
    slotmap::{SecondaryMap, SlotMap},
    util::cell::RefCell,
};
use tracing::{Span, error, instrument, trace};

pub extern crate serde_json;

mod ast;
mod cheats;

pub use cheats::add_cheats;

use crate::ast::Ast;

// TODO this should work with local variables too
#[derive(Debug)]
pub struct GlobalScope {
    constants: HashMap<&'static str, fn() -> Box<dyn EffectfulNode>>,
    global_vars: HashMap<Rc<str>, GlobalId>,
    patch_vars: HashMap<Rc<str>, PatchVarDesc>,
    turtle_vars: HashMap<Rc<str>, TurtleVarDesc>,
    /// The default turtle breed is represented by the empty string.
    turtle_breeds: HashMap<Rc<str>, BreedId>,
    functions: HashMap<Rc<str>, FunctionId>,
}

#[non_exhaustive]
#[derive(Debug)]
enum NameReferent {
    Constant(fn() -> Box<dyn EffectfulNode>),
    Global(GlobalId),
    TurtleVar(TurtleVarDesc),
    PatchVar(PatchVarDesc),
    TurtleBreed(BreedId),
    UserProc(FunctionId),
}

impl GlobalScope {
    fn with_builtins(default_turtle_breed: BreedId) -> Self {
        Self {
            constants: HashMap::from([
                (
                    "RED",
                    (|| Box::new(node::Constant { value: UnpackedDynBox::Float(15.0) }))
                        as fn() -> Box<dyn EffectfulNode>,
                ),
                ("ORANGE", || Box::new(node::Constant { value: UnpackedDynBox::Float(25.0) })),
                ("GREEN", || Box::new(node::Constant { value: UnpackedDynBox::Float(55.0) })),
                ("CYAN", || Box::new(node::Constant { value: UnpackedDynBox::Float(85.0) })),
                ("SKY", || Box::new(node::Constant { value: UnpackedDynBox::Float(95.0) })),
                ("BLUE", || Box::new(node::Constant { value: UnpackedDynBox::Float(105.0) })),
                ("VIOLET", || Box::new(node::Constant { value: UnpackedDynBox::Float(115.0) })),
                ("POPULATION", || Box::new(node::Constant { value: UnpackedDynBox::Int(125) })),
                ("DIFFUSION-RATE", || {
                    Box::new(node::Constant { value: UnpackedDynBox::Float(50.0) })
                }),
                ("EVAPORATION-RATE", || {
                    Box::new(node::Constant { value: UnpackedDynBox::Float(10.0) })
                }),
                ("TURTLES", || Box::new(node::Agentset::AllTurtles)),
                ("PATCHES", || Box::new(node::Agentset::AllPatches)),
            ]),
            global_vars: HashMap::new(),
            patch_vars: HashMap::from([(Rc::from("PCOLOR"), PatchVarDesc::Pcolor)]),
            turtle_vars: HashMap::from([
                (Rc::from("WHO"), TurtleVarDesc::Who),
                (Rc::from("COLOR"), TurtleVarDesc::Color),
                (Rc::from("SIZE"), TurtleVarDesc::Size),
            ]),
            turtle_breeds: HashMap::from([("".into(), default_turtle_breed)]),
            functions: HashMap::new(),
        }
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

#[derive(Debug)]
struct MirBuilder {
    globals: SlotMap<GlobalId, ()>,
    turtle_breeds: SlotMap<BreedId, ()>,
    functions: SlotMap<FunctionId, Function>,
    fn_info: SecondaryMap<FunctionId, FnInfo>,
    global_names: GlobalScope,
}

pub struct ParseResult {
    pub program: mir::Program,
    pub global_names: GlobalScope,
    pub fn_info: SecondaryMap<FunctionId, FnInfo>,
}

pub fn ast_to_mir(ast: Ast) -> anyhow::Result<ParseResult> {
    trace!("Starting AST to MIR conversion");

    let mut global_vars: SlotMap<GlobalId, ()> = SlotMap::with_key();
    let mut custom_patch_vars = Vec::new();
    let mut custom_turtle_vars = Vec::new();

    let mut turtle_breeds: SlotMap<BreedId, ()> = SlotMap::with_key();
    let default_turtle_breed = turtle_breeds.insert(());

    let mut global_scope = GlobalScope::with_builtins(default_turtle_breed);
    trace!("Initialized name scope with builtins");

    let Ast { global_names, procedures } = ast;

    let ast::GlobalNames {
        global_vars: global_var_names,
        turtle_vars: turtle_var_names,
        patch_vars: patch_var_names,
        link_vars: link_var_names,
    } = global_names;

    for global_name in global_var_names {
        let global_name: Rc<str> = Rc::from(global_name);
        let global_id = global_vars.insert(());
        trace!("Adding global variable `{}` with id {:?}", global_name, global_id);
        global_scope.global_vars.insert(global_name, global_id);
    }
    // add custom patch variables
    for (i, patch_var_name) in patch_var_names.into_iter().enumerate() {
        let patch_var_name: Rc<str> = Rc::from(patch_var_name);
        let patch_var_id = PatchVarDesc::Custom(i);
        trace!("Adding patch variable `{}` with id {:?}", patch_var_name, patch_var_id);
        global_scope.patch_vars.insert(patch_var_name.clone(), patch_var_id);
        custom_patch_vars
            .push(CustomVarDecl { name: patch_var_name, ty: NetlogoAbstractType::Top });
    }
    // add custom turtle variables
    for (i, turtle_var_name) in turtle_var_names.into_iter().enumerate() {
        let turtle_var_name: Rc<str> = Rc::from(turtle_var_name);
        let turtle_var_id = TurtleVarDesc::Custom(i);
        trace!("Adding turtle variable `{}` with id {:?}", turtle_var_name, turtle_var_id);
        global_scope.turtle_vars.insert(turtle_var_name.clone(), turtle_var_id);
        custom_turtle_vars
            .push(CustomVarDecl { name: turtle_var_name, ty: NetlogoAbstractType::Top });
    }

    let mut functions: SlotMap<FunctionId, Function> = SlotMap::with_key();
    let mut fn_info = SecondaryMap::new();
    let mut bodies_to_build = SecondaryMap::new();

    // go through each procedure and add a skeleton with just the signatures
    for procedure_ast in procedures {
        // compile for a set of hardcoded agent classes
        use ast::AgentClass as Ac;
        let agent_class = match procedure_ast.agent_class {
            // if any agent can execute it, it's probably the observer that executes it
            Ac { observer: true, turtle: true, patch: true, link: true } => AgentClass::Observer,
            Ac { observer: true, turtle: false, patch: false, link: false } => AgentClass::Observer,
            // -TP- means it uses patch variables, which is probably for patches
            // TODO account for the possibility that a turtle does execute it
            Ac { observer: false, turtle: true, patch: true, link: false } => AgentClass::Patch,
            Ac { observer: false, turtle: true, patch: false, link: false } => AgentClass::Turtle,
            _ => todo!("handle all agent classes"),
        };

        // create the skeleton
        trace!(
            "Creating procedure skeleton for `{}` (agent class: {:?})",
            procedure_ast.name, agent_class
        );
        let (procedure, signature, body) = create_procedure_skeleton(procedure_ast, agent_class)?;
        let proc_name = procedure.debug_name.as_ref().cloned().unwrap();
        let fn_id = functions.insert(procedure);
        fn_info.insert(fn_id, signature);
        global_scope.functions.insert(proc_name.clone(), fn_id);
        trace!("Created procedure skeleton for {} with function id: {:?}", proc_name, fn_id);

        // save the ast to build the body later
        bodies_to_build.insert(fn_id, body);
    }

    let mut mir_builder = MirBuilder {
        globals: global_vars,
        turtle_breeds,
        functions,
        fn_info,
        global_names: global_scope,
    };

    // then go through each procedure and build out the bodies
    for (fn_id, body) in bodies_to_build {
        let proc_name = mir_builder.functions[fn_id].debug_name.as_ref().cloned().unwrap();
        trace!("Building body for procedure: {} (id: {:?})", proc_name, fn_id);
        build_body(body, fn_id, &mut mir_builder)?;
        trace!("Completed body building for procedure: {}", proc_name);
    }

    Ok(ParseResult {
        program: mir::Program {
            globals: mir_builder.globals,
            turtle_breeds: mir_builder.turtle_breeds,
            custom_turtle_vars,
            custom_patch_vars,
            turtle_schema: None,
            patch_schema: None,
            functions: mir_builder
                .functions
                .into_iter()
                .map(|(id, function)| (id, RefCell::new(function)))
                .collect(),
        },
        global_names: mir_builder.global_names,
        fn_info: mir_builder.fn_info,
    })
}

#[derive(Debug, Copy, Clone)]
enum AgentClass {
    Observer,
    Turtle,
    Patch,
    Link,
    Any,
}

impl AgentClass {
    fn is_turtle(&self) -> bool {
        matches!(self, AgentClass::Turtle | AgentClass::Any)
    }

    fn is_patch(&self) -> bool {
        matches!(self, AgentClass::Patch | AgentClass::Any)
    }

    fn is_link(&self) -> bool {
        matches!(self, AgentClass::Link | AgentClass::Any)
    }
}

/// Holds information about a function while it is being built.
#[derive(Debug)]
pub struct FnInfo {
    env_param: Option<LocalId>,
    context_param: Option<LocalId>,
    self_param: Option<LocalId>,
    positional_params: Vec<LocalId>,
    local_names: HashMap<Rc<str>, LocalId>,
    num_internal_bodies: usize,
}

impl FnInfo {
    fn new() -> Self {
        Self {
            env_param: None,
            context_param: None,
            self_param: None,
            positional_params: Vec::new(),
            local_names: HashMap::new(),
            num_internal_bodies: 0,
        }
    }
}

#[instrument(
    skip(procedure_ast),
    fields(proc_name = procedure_ast.name),
)]
fn create_procedure_skeleton(
    procedure_ast: ast::Procedure,
    agent_class: AgentClass,
) -> anyhow::Result<(Function, FnInfo, Vec<ast::Node>)> {
    trace!("Creating procedure skeleton");
    let proc_name = Rc::from(procedure_ast.name);

    // verify that the procedure can support the given agent class
    match agent_class {
        #[allow(clippy::iter_nth_zero)]
        AgentClass::Observer => assert!(procedure_ast.agent_class.observer),
        AgentClass::Turtle => assert!(procedure_ast.agent_class.turtle),
        AgentClass::Patch => assert!(procedure_ast.agent_class.patch),
        AgentClass::Link => assert!(procedure_ast.agent_class.link),
        AgentClass::Any => {}
    }

    // calculate the function parameters
    let mut fn_info = FnInfo::new();
    let mut locals = SlotMap::with_key();
    let mut parameter_locals = Vec::new();
    // always add the context parameter TODO this shouldn't be always
    let context_param = locals.insert(LocalDeclaration {
        debug_name: Some("context".into()),
        ty: MirType::Machine(NetlogoMachineType::UNTYPED_PTR),
        storage: LocalStorage::Register,
    });
    parameter_locals.push(context_param);
    fn_info.context_param = Some(context_param);
    trace!("Added context parameter with local_id: {:?}", context_param);
    match agent_class {
        AgentClass::Observer => {
            trace!("No self parameter needed for Observer agent class");
        }
        AgentClass::Turtle => {
            let local_id = locals.insert(LocalDeclaration {
                debug_name: Some("self_turtle_id".into()),
                ty: MirType::Machine(NetlogoMachineType::TURTLE_ID),
                storage: LocalStorage::Register,
            });
            parameter_locals.push(local_id);
            fn_info.self_param = Some(local_id);
            trace!("Added turtle self parameter with local_id: {:?}", local_id);
        }
        AgentClass::Patch => {
            let local_id = locals.insert(LocalDeclaration {
                debug_name: Some("self_patch_id".into()),
                ty: MirType::Machine(NetlogoMachineType::PATCH_ID),
                storage: LocalStorage::Register,
            });
            parameter_locals.push(local_id);
            fn_info.self_param = Some(local_id);
            trace!("Added patch self parameter with local_id: {:?}", local_id);
        }
        AgentClass::Link => todo!(),
        AgentClass::Any => todo!(),
    }
    for arg_name in procedure_ast.arg_names {
        let arg_name: Rc<str> = Rc::from(arg_name);
        let local_id = locals.insert(LocalDeclaration {
            debug_name: Some(arg_name.clone()),
            ty: MirType::Abstract(NetlogoAbstractType::Top),
            storage: LocalStorage::Register,
        });
        trace!("Adding positional parameter {} with local_id: {:?}", arg_name, local_id);
        fn_info.local_names.insert(arg_name, local_id);
        parameter_locals.push(local_id);
        fn_info.positional_params.push(local_id);
    }

    // calculate the function return type
    let return_ty = match procedure_ast.return_type {
        ast::ReturnType::Unit => NetlogoAbstractType::Unit,
        ast::ReturnType::Wildcard => NetlogoAbstractType::Top,
    };
    trace!("calculated return type: {:?}", return_ty);

    // create the function skeleton
    let function = Function {
        debug_name: Some(proc_name),
        parameters: parameter_locals,
        return_ty,
        locals,
        // cfg and nodes are defaulted values and will be filled in later
        cfg: StatementBlock { statements: vec![] },
        nodes: RefCell::new(SlotMap::with_key()),
    };
    trace!(
        "Created function skeleton for {} with {} parameters",
        function.debug_name.as_deref().unwrap(),
        function.parameters.len()
    );

    Ok((function, fn_info, procedure_ast.statements))
}

/// # Arguments
///
/// * `function_id`: The id of the function whose body is being written
#[instrument(skip(statements_ast, mir_builder))]
fn build_body(
    statements_ast: Vec<ast::Node>,
    fn_id: FunctionId,
    mir_builder: &mut MirBuilder,
) -> anyhow::Result<()> {
    trace!("building body");

    // the nodes for this function
    let mut nodes: SlotMap<NodeId, Box<dyn EffectfulNode>> = SlotMap::with_key();

    // the statements of the current control flow construct
    let statements: Vec<StatementKind> = statements_ast
        .into_iter()
        .map(|stmt_ast| {
            let ctx = FnBodyBuilderCtx { mir: mir_builder, fn_id, nodes: &mut nodes };
            translate_statement(stmt_ast, ctx)
        })
        .collect();

    let function = &mut mir_builder.functions[fn_id];
    function.cfg = StatementBlock { statements };
    function.nodes = RefCell::new(nodes);

    trace!("finished building body");
    Ok(())
}

struct FnBodyBuilderCtx<'a> {
    mir: &'a mut MirBuilder,
    fn_id: FunctionId,
    nodes: &'a mut SlotMap<NodeId, Box<dyn EffectfulNode>>,
}

impl<'a> FnBodyBuilderCtx<'a> {
    fn reborrow<'s>(&'s mut self) -> FnBodyBuilderCtx<'s> {
        FnBodyBuilderCtx { mir: self.mir, fn_id: self.fn_id, nodes: self.nodes }
    }

    /// Returns a node that gets the context parameter for the current function
    fn get_context(&mut self) -> NodeId {
        let id = self.nodes.insert(Box::new(node::GetLocalVar {
            local_id: self.mir.fn_info[self.fn_id].context_param.unwrap(),
        }));
        trace!("Got context parameter with id: {:?}", id);
        id
    }

    /// Returns a node that gets the turtle self parameter. Panics if the current
    /// function doesn't have a turtle self parameter.
    fn get_self_turtle(&mut self) -> NodeId {
        let self_param = self.mir.fn_info[self.fn_id].self_param.unwrap();
        // assert!(agent_class.is_turtle(), "Expected turtle agent class");
        let id = self.nodes.insert(Box::new(node::GetLocalVar { local_id: self_param }));
        trace!("Got turtle self parameter with id: {:?}", id);
        id
    }

    /// Returns a node that gets the patch self parameter. Panics if the current
    /// function doesn't have a patch self parameter.
    fn get_self_patch(&mut self) -> NodeId {
        let self_param = self.mir.fn_info[self.fn_id].self_param.unwrap();
        // assert!(agent_class.is_patch(), "Expected patch agent class");
        self.nodes.insert(Box::new(node::GetLocalVar { local_id: self_param }))
    }

    /// Returns a node that gets the self parameter for turtle or patch agents.
    /// Panics if the current function doesn't have a turtle or patch self parameter.
    fn get_self_turtle_or_patch(&mut self) -> NodeId {
        let self_param = self.mir.fn_info[self.fn_id].self_param.unwrap();
        // assert!(
        //     agent_class.is_turtle() || agent_class.is_patch(),
        //     "Expected turtle or patch agent class"
        // );
        self.nodes.insert(Box::new(node::GetLocalVar { local_id: self_param }))
    }

    /// Returns a node that gets the self parameter for any agent type.
    fn get_self_any(&mut self) -> NodeId {
        let self_param = self.mir.fn_info[self.fn_id].self_param.unwrap();
        self.nodes.insert(Box::new(node::GetLocalVar { local_id: self_param }))
    }
}

#[instrument(skip_all, fields(node_type, name))]
fn translate_statement(ast_node: ast::Node, mut ctx: FnBodyBuilderCtx<'_>) -> StatementKind {
    use ast::CommandApp as C;
    use ast::Node as N;
    let mir_node: Box<dyn EffectfulNode> = match ast_node {
        N::LetBinding { var_name, value } => {
            return translate_let_binding(Rc::from(var_name.as_str()), *value, ctx.reborrow());
        }
        N::CommandApp(C::UserProcCall { name, args }) => {
            let referent = ctx
                .mir
                .global_names
                .lookup(&name)
                .unwrap_or_else(|| panic!("unknown command {:?}", name));
            let NameReferent::UserProc(target) = referent else {
                panic!("expected a user procedure, got {:?}", referent);
            };
            let args =
                args.into_iter().map(|arg| translate_expression(arg, ctx.reborrow())).collect();
            Box::new(node::CallUserFn { target, args })
        }
        N::CommandApp(C::Report([value])) => {
            let value = translate_expression(*value, ctx.reborrow());
            return StatementKind::Return { value };
        }
        N::CommandApp(C::Stop([])) => return StatementKind::Stop,
        N::CommandApp(C::ClearAll([])) => {
            let context = ctx.get_context();
            Box::new(node::ClearAll { context })
        }
        N::CommandApp(C::CreateTurtles([population, body])) => {
            let context = ctx.get_context();
            let population = translate_expression(*population, ctx.reborrow());
            let body =
                translate_ephemeral_closure(*body, ctx.fn_id, AgentClass::Turtle, ctx.reborrow());
            Box::new(node::CreateTurtles {
                context,
                breed: ctx.mir.global_names.turtle_breeds[""], // FUTURE add creating other turtle breeds
                num_turtles: population,
                body,
            })
        }
        N::CommandApp(C::Set([var, value])) => {
            let var_name = translate_var_reporter_without_read(var.as_ref());
            let var_desc = ctx.mir.global_names.lookup(var_name).unwrap();
            let value = translate_expression(*value, ctx.reborrow());

            // the kind of variable being  assigned determines which node to use
            match var_desc {
                NameReferent::TurtleVar(var) => {
                    let context = ctx.get_context();
                    let turtle = ctx.get_self_turtle();
                    Box::new(node::SetTurtleVar { context, turtle, var, value })
                }
                NameReferent::PatchVar(var) => {
                    let context = ctx.get_context();
                    let agent = ctx.get_self_turtle_or_patch();
                    Box::new(node::SetPatchVarAsTurtleOrPatch { context, agent, var, value })
                }
                NameReferent::Global(_) => todo!("setting global variables not yet supported"),
                other => panic!("cannot mutate value of {:?}", other),
            }
        }
        N::CommandApp(C::Fd([distance])) => {
            let context = ctx.get_context();
            let turtle = ctx.get_self_turtle();
            let distance = translate_expression(*distance, ctx.reborrow());
            Box::new(node::TurtleForward { context, turtle, distance })
        }
        N::CommandApp(C::Left([heading])) => {
            let context = ctx.get_context();
            let turtle = ctx.get_self_turtle();
            let angle_rt = translate_expression(*heading, ctx.reborrow());
            let angle_lt = ctx
                .nodes
                .insert(Box::new(node::UnaryOp { op: UnaryOpcode::Neg, operand: angle_rt }));
            Box::new(node::TurtleRotate { context, turtle, angle: angle_lt })
        }
        N::CommandApp(C::Right([heading])) => {
            let context = ctx.get_context();
            let turtle = ctx.get_self_turtle();
            let angle = translate_expression(*heading, ctx.reborrow());
            Box::new(node::TurtleRotate { context, turtle, angle })
        }
        N::CommandApp(C::ResetTicks([])) => {
            Box::new(node::ResetTicks { context: ctx.get_context() })
        }
        N::CommandApp(C::Ask([recipients, body])) => {
            let context = ctx.get_context();
            let recipients = translate_expression(*recipients, ctx.reborrow());
            let body =
                translate_ephemeral_closure(*body, ctx.fn_id, AgentClass::Any, ctx.reborrow());
            Box::new(node::Ask { context, recipients, body })
        }
        N::CommandApp(C::If([condition, then_block])) => {
            let condition = translate_expression(*condition, ctx.reborrow());
            let then_block = translate_statement_block(*then_block, ctx.reborrow());
            return StatementKind::IfElse {
                condition,
                then_block,
                else_block: StatementBlock::default(),
            };
        }
        N::CommandApp(C::IfElse([condition, then_block, else_block])) => {
            let condition = translate_expression(*condition, ctx.reborrow());
            let then_block = translate_statement_block(*then_block, ctx.reborrow());
            let else_block = translate_statement_block(*else_block, ctx.reborrow());
            return StatementKind::IfElse { condition, then_block, else_block };
        }
        N::CommandApp(C::Diffuse([variable, amt])) => {
            let var_name = translate_var_reporter_without_read(variable.as_ref());
            let Some(NameReferent::PatchVar(var_desc)) = ctx.mir.global_names.lookup(var_name)
            else {
                panic!("expected patch variable for DIFFUSE");
            };
            let context = ctx.get_context();
            let amt = translate_expression(*amt, ctx.reborrow());
            Box::new(node::Diffuse { context, variable: var_desc, amt })
        }
        N::CommandApp(C::Tick([])) => Box::new(node::AdvanceTick { context: ctx.get_context() }),
        N::CommandApp(C::SetDefaultShape([breed, shape])) => {
            let breed = translate_expression(*breed, ctx.reborrow());
            let shape = translate_expression(*shape, ctx.reborrow());
            Box::new(node::SetDefaultShape { breed, shape })
        }
        other => panic!("expected a statement, got {:?}", other),
    };
    // most statements are just StatementKind::Node. there will be an early
    // return in the big match above if it is a different kind
    StatementKind::Node(ctx.nodes.insert(mir_node))
}

fn translate_expression(expr: ast::Node, mut ctx: FnBodyBuilderCtx<'_>) -> NodeId {
    use ast::Node as N;
    use ast::ReporterCall as R;
    let mir_node: Box<dyn EffectfulNode> = match expr {
        N::LetRef { name } | N::ProcedureArgRef { name } => {
            let Some(&local_id) = ctx.mir.fn_info[ctx.fn_id].local_names.get(name.as_str()) else {
                unreachable!("unknown variable reference: {}", name);
            };
            Box::new(node::GetLocalVar { local_id })
        }
        N::Number { value } => {
            Box::new(node::Constant { value: UnpackedDynBox::Float(value.as_f64().unwrap()) })
        }
        N::String { value } => {
            // TODO implement string constants
            Box::new(node::Constant { value: UnpackedDynBox::Float(0.0) })
        }
        N::List { items } => todo!(),
        N::Nobody => Box::new(node::Constant { value: UnpackedDynBox::Nobody }),
        N::ReporterProcCall { name, args } => {
            let referent = ctx.mir.global_names.lookup(&name).unwrap_or_else(|| {
                panic!("unknown reporter procedure {:?}", name);
            });
            let NameReferent::UserProc(target) = referent else {
                panic!("expected a user reporter procedure, got {:?}", referent);
            };
            let args =
                args.into_iter().map(|arg| translate_expression(arg, ctx.reborrow())).collect();
            Box::new(node::CallUserFn { target, args })
        }
        N::ReporterCall(R::VarAccess { name }) => {
            let Some(referent) = ctx.mir.global_names.lookup(&name) else {
                panic!("unknown variable access `{}`", name);
            };
            match referent {
                NameReferent::Global(global_id) => {
                    todo!("global variable accesses not yet implemented")
                }
                NameReferent::TurtleVar(var) => {
                    let context = ctx.get_context();
                    let turtle = ctx.get_self_turtle();
                    Box::new(node::GetTurtleVar { context, turtle, var })
                }
                NameReferent::PatchVar(var) => {
                    let context = ctx.get_context();
                    let agent = ctx.get_self_turtle_or_patch();
                    Box::new(node::GetPatchVarAsTurtleOrPatch { context, agent, var })
                }
                NameReferent::Constant(mk_node) => mk_node(),
                _ => panic!("unexpected variable referent {:?} for name {}", referent, name),
            }
        }
        N::ReporterCall(R::Of([body, recipients])) => {
            let context = ctx.get_context();
            let recipients = translate_expression(*recipients, ctx.reborrow());
            let body =
                translate_ephemeral_closure(*body, ctx.fn_id, AgentClass::Any, ctx.reborrow());
            Box::new(node::Of { context, recipients, body })
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
            let lhs = translate_expression(*lhs, ctx.reborrow());
            let rhs = translate_expression(*rhs, ctx.reborrow());
            Box::new(node::BinaryOperation { op, lhs, rhs })
        }
        N::ReporterCall(R::Not([operand])) => {
            let operand = translate_expression(*operand, ctx.reborrow());
            Box::new(node::UnaryOp { op: UnaryOpcode::Not, operand })
        }
        N::ReporterCall(R::Distancexy([x, y])) => {
            let agent = ctx.get_self_turtle_or_patch();
            let x = translate_expression(*x, ctx.reborrow());
            let y = translate_expression(*y, ctx.reborrow());
            Box::new(node::Distancexy { agent, x, y })
        }
        N::ReporterCall(R::CanMove([distance])) => {
            let context = ctx.get_context();
            let turtle = ctx.get_self_turtle();
            let distance = translate_expression(*distance, ctx.reborrow());
            Box::new(node::CanMove { context, turtle, distance })
        }
        N::ReporterCall(
            reporter @ (R::PatchRightAndAhead { .. } | R::PatchLeftAndAhead { .. }),
        ) => {
            let (relative_loc, heading, distance) = match reporter {
                R::PatchRightAndAhead([heading, distance]) => {
                    (PatchLocRelation::RightAhead, heading, distance)
                }
                R::PatchLeftAndAhead([heading, distance]) => {
                    (PatchLocRelation::LeftAhead, heading, distance)
                }
                _ => unreachable!(),
            };
            let context = ctx.get_context();
            let turtle = ctx.get_self_turtle();
            let heading = translate_expression(*heading, ctx.reborrow());
            let distance = translate_expression(*distance, ctx.reborrow());
            Box::new(node::PatchRelative { context, turtle, relative_loc, heading, distance })
        }
        N::ReporterCall(R::MaxPxcor([])) => Box::new(node::MaxPxcor { context: ctx.get_context() }),
        N::ReporterCall(R::MaxPycor([])) => Box::new(node::MaxPycor { context: ctx.get_context() }),
        N::ReporterCall(R::OneOf(choices)) => {
            Box::new(node::Constant { value: UnpackedDynBox::Float(1.0) })
        }
        N::ReporterCall(R::ScaleColor([color, number, range1, range2])) => {
            let color = translate_expression(*color, ctx.reborrow());
            let number = translate_expression(*number, ctx.reborrow());
            let range1 = translate_expression(*range1, ctx.reborrow());
            let range2 = translate_expression(*range2, ctx.reborrow());
            Box::new(node::ScaleColor { color, number, range1, range2 })
        }
        N::ReporterCall(R::Ticks([])) => Box::new(node::GetTick { context: ctx.get_context() }),
        N::ReporterCall(R::Random([bound])) => {
            let bound = translate_expression(*bound, ctx.reborrow());
            Box::new(node::RandomInt { context: ctx.get_context(), bound })
        }
        other => panic!("expected an expression, got {:?}", other),
    };
    ctx.nodes.insert(mir_node)
}

fn translate_statement_block(
    statements_ast: ast::Node,
    mut ctx: FnBodyBuilderCtx<'_>,
) -> StatementBlock {
    let ast::Node::CommandBlock { statements } = statements_ast else {
        panic!("expected a command block, got {:?}", statements_ast);
    };
    let statements = statements
        .into_iter()
        .map(|ast_node| translate_statement(ast_node, ctx.reborrow()))
        .collect();
    StatementBlock { statements }
}

fn translate_let_binding(
    name: Rc<str>,
    value: ast::Node,
    mut ctx: FnBodyBuilderCtx<'_>,
) -> StatementKind {
    let local_id = ctx.mir.functions[ctx.fn_id].locals.insert(LocalDeclaration {
        debug_name: Some(name.clone()),
        ty: MirType::Abstract(NetlogoAbstractType::Top),
        storage: LocalStorage::Register,
    });
    ctx.mir.fn_info[ctx.fn_id].local_names.insert(name, local_id);
    let value = translate_expression(value, ctx.reborrow());
    StatementKind::Node(ctx.nodes.insert(Box::new(node::SetLocalVar { local_id, value })))
}

fn translate_var_reporter_without_read(ast_node: &ast::Node) -> &str {
    let ast::Node::ReporterCall(ast::ReporterCall::VarAccess { name }) = ast_node else {
        panic!("expected a variable reporter call, got {:?}", ast_node);
    };
    name
}

#[instrument(skip_all)]
fn translate_ephemeral_closure(
    expr: ast::Node,
    parent_fn_id: FunctionId,
    agent_class: AgentClass,
    ctx: FnBodyBuilderCtx<'_>,
) -> NodeId {
    trace!("Translating ephemeral closure");

    use ast::Node as N;
    let (statements, return_ty) = match expr {
        N::CommandBlock { statements } => (statements, NetlogoAbstractType::Unit),
        N::ReporterBlock { reporter_app } => {
            let statements = vec![N::CommandApp(ast::CommandApp::Report([reporter_app]))];
            (statements, NetlogoAbstractType::Top)
        }
        _ => panic!("expected a command block or reporter block, got {:?}", expr),
    };

    // generate a procedure name
    let parent_fn_bodies = &mut ctx.mir.fn_info[parent_fn_id].num_internal_bodies;
    let parent_fn_name = ctx.mir.functions[parent_fn_id].debug_name.as_deref().unwrap();
    let proc_name = Rc::from(format!("{} body {}", parent_fn_name, *parent_fn_bodies));
    *parent_fn_bodies += 1;

    // calculate the function parameters
    let mut fn_info = FnInfo::new();
    let mut locals = SlotMap::with_key();
    let mut parameter_locals = Vec::new();

    // add the environment pointer
    let env_param = locals.insert(LocalDeclaration {
        debug_name: Some("env".into()),
        ty: MirType::Machine(NetlogoMachineType::UNTYPED_PTR),
        storage: LocalStorage::Register,
    });
    parameter_locals.push(env_param);
    fn_info.env_param = Some(env_param);

    // add the context parameter
    let context_param = locals.insert(LocalDeclaration {
        debug_name: Some("context".into()),
        ty: MirType::Machine(NetlogoMachineType::UNTYPED_PTR),
        storage: LocalStorage::Register,
    });
    parameter_locals.push(context_param);
    fn_info.context_param = Some(context_param);

    // add the self parameter
    match agent_class {
        AgentClass::Observer => {
            trace!("No self parameter needed for Observer agent class");
        }
        AgentClass::Turtle => {
            let local_id = locals.insert(LocalDeclaration {
                debug_name: Some("self_turtle_id".into()),
                ty: MirType::Machine(NetlogoMachineType::TURTLE_ID),
                storage: LocalStorage::Register,
            });
            parameter_locals.push(local_id);
            fn_info.self_param = Some(local_id);
            trace!("Added turtle self parameter with local_id: {:?}", local_id);
        }
        AgentClass::Patch => {
            let local_id = locals.insert(LocalDeclaration {
                debug_name: Some("self_patch_id".into()),
                ty: MirType::Machine(NetlogoMachineType::PATCH_ID),
                storage: LocalStorage::Register,
            });
            parameter_locals.push(local_id);
            fn_info.self_param = Some(local_id);
            trace!("Added patch self parameter with local_id: {:?}", local_id);
        }
        AgentClass::Link => todo!(),
        AgentClass::Any => {
            let local_id = locals.insert(LocalDeclaration {
                debug_name: Some("self_any".into()),
                ty: MirType::Abstract(NetlogoAbstractType::Top),
                storage: LocalStorage::Register,
            });
            parameter_locals.push(local_id);
            fn_info.self_param = Some(local_id);
            trace!("Added any self parameter with local_id: {:?}", local_id);
        }
    }

    // create the function skeleton
    let function = Function {
        debug_name: Some(proc_name),
        parameters: parameter_locals,
        return_ty,
        locals,
        // cfg and nodes are defaulted and will be filled in later
        cfg: StatementBlock::default(),
        nodes: RefCell::default(),
    };
    let fn_id = ctx.mir.functions.insert(function);
    ctx.mir.fn_info.insert(fn_id, fn_info);
    trace!("Inserted function for closure with id: {:?}", fn_id);

    // build the function body
    build_body(statements, fn_id, ctx.mir).unwrap();

    // return a closure object
    ctx.nodes.insert(Box::new(node::Closure {
        captures: vec![], // TODO add captures
        body: fn_id,
    }))
}

#[instrument(skip_all)]
pub fn write_dot(fn_id: FunctionId, function: &Function) {
    let dot_string = function.to_dot_string_with_options(false);
    let filename =
        format!("dots/{}-{:?}.dot", fn_id, function.debug_name.as_deref().unwrap_or("unnamed"));
    trace!("Writing DOT file for function {:?}: {}", fn_id, filename);

    if let Some(parent) = Path::new(&filename).parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            panic!("Failed to create parent directory for {} | {:?}", filename, e);
        }
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

        for (fn_id, function) in program.functions {
            write_dot(fn_id, &*function.borrow());
        }
    }
}
