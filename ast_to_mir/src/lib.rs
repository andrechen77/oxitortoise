#![feature(if_let_guard)]

use std::{collections::HashMap, rc::Rc};

use engine::{
    mir::{
        self, CustomVarDecl, EffectfulNode, Function, FunctionId, GlobalId, LocalDeclaration,
        LocalId, LocalStorage, MirType, NetlogoAbstractType, NodeId, StatementBlock, StatementKind,
        node::{self, BinaryOpcode, PatchLocRelation, UnaryOpcode},
    },
    sim::{
        patch::PatchVarDesc,
        turtle::{BreedId, TurtleVarDesc},
        value::{NetlogoMachineType, UnpackedDynBox},
    },
    slotmap::{SecondaryMap, SlotMap},
    util::cell::RefCell,
};
use serde_json::{Value as JsonValue, json};
use tracing::{Span, error, instrument, trace};

pub extern crate serde_json;

type JsonObj = serde_json::Map<String, JsonValue>;

// TODO this should work with local variables too
#[derive(Debug)]
struct GlobalNames {
    constants: HashMap<&'static str, fn() -> Box<dyn EffectfulNode>>,
    globals: HashMap<Rc<str>, GlobalId>,
    patch_vars: HashMap<Rc<str>, PatchVarDesc>,
    turtle_vars: HashMap<Rc<str>, TurtleVarDesc>,
    turtle_breeds: HashMap<Option<Rc<str>>, BreedId>,
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

impl GlobalNames {
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
            globals: HashMap::new(),
            patch_vars: HashMap::from([(Rc::from("PCOLOR"), PatchVarDesc::Pcolor)]),
            turtle_vars: HashMap::from([
                (Rc::from("WHO"), TurtleVarDesc::Who),
                (Rc::from("COLOR"), TurtleVarDesc::Color),
                (Rc::from("SIZE"), TurtleVarDesc::Size),
            ]),
            turtle_breeds: HashMap::from([(None, default_turtle_breed)]),
            functions: HashMap::new(),
        }
    }

    fn lookup(&self, name: &str) -> Option<NameReferent> {
        let Self { constants, globals, patch_vars, turtle_vars, turtle_breeds, functions } = self;
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
        // idk how to get lookup by option working
        // if let Some(turtle_breed_id) = turtle_breeds.get(Some(name)) {
        //     return Some(NameReferent::TurtleBreed(*turtle_breed_id));
        // }
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
    global_names: GlobalNames,
}

pub fn ast_to_mir(ast: &JsonValue) -> anyhow::Result<mir::Program> {
    trace!("Starting AST to MIR conversion");
    let root = ast.as_object().unwrap();

    let mut globals: SlotMap<GlobalId, ()> = SlotMap::with_key();
    let mut custom_patch_vars = Vec::new();
    let mut custom_turtle_vars = Vec::new();

    let mut turtle_breeds: SlotMap<BreedId, ()> = SlotMap::with_key();
    let default_turtle_breed = turtle_breeds.insert(());

    let mut name_scope = GlobalNames::with_builtins(default_turtle_breed);
    trace!("Initialized name scope with builtins");

    let meta_vars = root["metaVars"].as_object().unwrap();
    // add global variables
    for global in meta_vars["globals"].as_array().unwrap() {
        let global_name = global.as_str().unwrap();
        let global_id = globals.insert(());
        name_scope.globals.insert(Rc::from(global_name), global_id);
        trace!("Added global variable `{}` with id {:?}", global_name, global_id);
    }
    // add custom patch variables
    for (i, patch_var) in meta_vars["patchVars"].as_array().unwrap().iter().enumerate() {
        let patch_var_name = patch_var.as_str().unwrap();
        let patch_var_id = PatchVarDesc::Custom(i);
        name_scope.patch_vars.insert(Rc::from(patch_var_name), patch_var_id);
        custom_patch_vars
            .push(CustomVarDecl { name: Rc::from(patch_var_name), ty: NetlogoAbstractType::Top });
        trace!("Added patch variable `{}` with id {:?}", patch_var_name, patch_var_id);
    }
    // add custom turtle variables
    for (i, turtle_var) in meta_vars["turtleVars"].as_array().unwrap().iter().enumerate() {
        let turtle_var_name = turtle_var.as_str().unwrap();
        let turtle_var_id = TurtleVarDesc::Custom(i);
        name_scope.turtle_vars.insert(Rc::from(turtle_var_name), turtle_var_id);
        custom_turtle_vars
            .push(CustomVarDecl { name: Rc::from(turtle_var_name), ty: NetlogoAbstractType::Top });
        trace!("Added turtle variable `{}` with id {:?}", turtle_var_name, turtle_var_id);
    }

    let mut functions: SlotMap<FunctionId, Function> = SlotMap::with_key();
    let mut fn_info = SecondaryMap::new();
    let mut functions_to_build = SecondaryMap::new();

    // go through each procedure and add a skeleton with just the signatures
    for procedure_json in
        root["procedures"].as_array().unwrap().iter().map(|p| p.as_object().unwrap())
    {
        let proc_name = procedure_json["name"].as_str().unwrap();

        // compile for a set of hardcoded agent classes
        let agent_class = match procedure_json["agentClass"].as_str().unwrap() {
            // if any agent can execute it, it's probably the observer that executes it
            "OTPL" => AgentClass::Observer,
            "O---" => AgentClass::Observer,
            // -TP- means it uses patch variables, which is probably for patches
            "-TP-" => AgentClass::Patch,
            "-T--" => AgentClass::Turtle,
            _ => todo!("handle all agent classes"),
        };

        // create the skeleton
        trace!("Creating procedure skeleton for `{}` (agent class: {:?})", proc_name, agent_class);
        let (procedure, signature) = create_procedure_skeleton(procedure_json, agent_class)?;
        let fn_id = functions.insert(procedure);
        fn_info.insert(fn_id, signature);
        name_scope.functions.insert(Rc::from(proc_name), fn_id);
        trace!("Created procedure skeleton for {} with function id: {:?}", proc_name, fn_id);

        // save the json to build the body later
        functions_to_build.insert(fn_id, procedure_json);
    }

    let mut mir_builder =
        MirBuilder { globals, turtle_breeds, functions, fn_info, global_names: name_scope };

    // then go through each procedure and build out the bodies
    for (function_id, procedure_json) in functions_to_build {
        let proc_name = procedure_json["name"].as_str().unwrap();
        trace!("Building body for procedure: {} (id: {:?})", proc_name, function_id);
        build_body(
            procedure_json["statements"].as_array().unwrap(),
            function_id,
            &mut mir_builder,
        )?;
        trace!("Completed body building for procedure: {}", proc_name);
    }

    Ok(mir::Program {
        globals: mir_builder.globals,
        turtle_breeds: mir_builder.turtle_breeds,
        custom_turtle_vars,
        custom_patch_vars,
        turtle_schema: None,
        functions: mir_builder
            .functions
            .into_iter()
            .map(|(id, function)| (id, RefCell::new(function)))
            .collect(),
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
struct FnInfo {
    /// The name of the entire function.
    debug_name: Rc<str>,
    env_param: Option<LocalId>,
    context_param: Option<LocalId>,
    self_param: Option<(AgentClass, LocalId)>,
    positional_params: Vec<LocalId>,
    local_names: HashMap<Rc<str>, LocalId>,
    num_internal_bodies: usize,
}

impl FnInfo {
    fn new(debug_name: Rc<str>) -> Self {
        Self {
            debug_name,
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
    skip(procedure),
    fields(proc_name = procedure["name"].as_str().unwrap()),
)]
fn create_procedure_skeleton(
    procedure: &JsonObj,
    agent_class: AgentClass,
) -> anyhow::Result<(Function, FnInfo)> {
    trace!("Creating procedure skeleton");
    let proc_name = procedure["name"].as_str().unwrap();

    // verify that the procedure can support the given agent class
    let agent_class_str = procedure["agentClass"].as_str().unwrap();
    match agent_class {
        AgentClass::Observer => assert_eq!(agent_class_str.chars().nth(0).unwrap(), 'O',),
        AgentClass::Turtle => assert_eq!(agent_class_str.chars().nth(1).unwrap(), 'T',),
        AgentClass::Patch => assert_eq!(agent_class_str.chars().nth(2).unwrap(), 'P',),
        AgentClass::Link => assert_eq!(agent_class_str.chars().nth(3).unwrap(), 'L',),
        AgentClass::Any => {}
    }

    // calculate the function parameters
    let mut fn_info = FnInfo::new(Rc::from(proc_name));
    let mut locals = SlotMap::with_key();
    let mut parameter_locals = Vec::new();
    // always add the context parameter TODO this shouldn't be always
    let context_param = locals.insert(LocalDeclaration {
        debug_name: Some("context".to_string()),
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
                debug_name: Some("self_turtle_id".to_string()),
                ty: MirType::Machine(NetlogoMachineType::TURTLE_ID),
                storage: LocalStorage::Register,
            });
            parameter_locals.push(local_id);
            fn_info.self_param = Some((AgentClass::Turtle, local_id));
            trace!("Added turtle self parameter with local_id: {:?}", local_id);
        }
        AgentClass::Patch => {
            let local_id = locals.insert(LocalDeclaration {
                debug_name: Some("self_patch_id".to_string()),
                ty: MirType::Machine(NetlogoMachineType::PATCH_ID),
                storage: LocalStorage::Register,
            });
            parameter_locals.push(local_id);
            fn_info.self_param = Some((AgentClass::Patch, local_id));
            trace!("Added patch self parameter with local_id: {:?}", local_id);
        }
        AgentClass::Link => todo!(),
        AgentClass::Any => todo!(),
    }
    for arg in procedure["args"].as_array().unwrap().iter().map(|arg| arg.as_str().unwrap()) {
        let local_id = locals.insert(LocalDeclaration {
            debug_name: Some(arg.to_string()),
            ty: MirType::Abstract(NetlogoAbstractType::Top),
            storage: LocalStorage::Register,
        });
        fn_info.local_names.insert(Rc::from(arg), local_id);
        parameter_locals.push(local_id);
        fn_info.positional_params.push(local_id);
        trace!("Added positional parameter {} with local_id: {:?}", arg, local_id);
    }

    // calculate the function return type
    let return_ty = match procedure["returnType"].as_str().unwrap() {
        "unit" => NetlogoAbstractType::Unit,
        "wildcard" => NetlogoAbstractType::Top,
        _ => todo!(),
    };
    trace!("calculated return type: {:?}", return_ty);

    // create the function skeleton
    let function = Function {
        debug_name: Some(proc_name.to_string()),
        parameters: parameter_locals,
        return_ty,
        locals,
        // cfg and nodes are defaulted values and will be filled in later
        cfg: StatementBlock { statements: vec![] },
        nodes: RefCell::new(SlotMap::with_key()),
    };
    trace!(
        "Created function skeleton for {} with {} parameters",
        proc_name,
        function.parameters.len()
    );

    Ok((function, fn_info))
}

/// # Arguments
///
/// * `function_id`: The id of the function whose body is being written
#[instrument(skip(statements_json, mir_builder))]
fn build_body(
    statements_json: &[JsonValue],
    fn_id: FunctionId,
    mir_builder: &mut MirBuilder,
) -> anyhow::Result<()> {
    trace!("building body");

    // the nodes for this function
    let mut nodes: SlotMap<NodeId, Box<dyn EffectfulNode>> = SlotMap::with_key();

    // the statements of the current control flow construct
    let statements: Vec<StatementKind> = statements_json
        .iter()
        .filter_map(|stmt_json| {
            let statement_json = stmt_json.as_object().unwrap();
            let ctx = FnBodyBuilderCtx { mir: mir_builder, fn_id, nodes: &mut nodes };
            match statement_json["type"].as_str().unwrap() {
                "let-binding" => eval_let_binding(statement_json, ctx),
                "command-app" => eval_command(statement_json, ctx),
                _ => todo!(),
            }
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
        let (agent_class, self_param) = self.mir.fn_info[self.fn_id].self_param.unwrap();
        assert!(agent_class.is_turtle(), "Expected turtle agent class");
        let id = self.nodes.insert(Box::new(node::GetLocalVar { local_id: self_param }));
        trace!("Got turtle self parameter with id: {:?}", id);
        id
    }

    /// Returns a node that gets the patch self parameter. Panics if the current
    /// function doesn't have a patch self parameter.
    fn get_self_patch(&mut self) -> NodeId {
        let (agent_class, self_param) = self.mir.fn_info[self.fn_id].self_param.unwrap();
        assert!(agent_class.is_patch(), "Expected patch agent class");
        self.nodes.insert(Box::new(node::GetLocalVar { local_id: self_param }))
    }

    /// Returns a node that gets the self parameter for turtle or patch agents.
    /// Panics if the current function doesn't have a turtle or patch self parameter.
    fn get_self_turtle_or_patch(&mut self) -> NodeId {
        let (agent_class, self_param) = self.mir.fn_info[self.fn_id].self_param.unwrap();
        assert!(
            agent_class.is_turtle() || agent_class.is_patch(),
            "Expected turtle or patch agent class"
        );
        self.nodes.insert(Box::new(node::GetLocalVar { local_id: self_param }))
    }

    /// Returns a node that gets the self parameter for any agent type.
    fn get_self_any(&mut self) -> NodeId {
        let (_agent_class, self_param) = self.mir.fn_info[self.fn_id].self_param.unwrap();
        self.nodes.insert(Box::new(node::GetLocalVar { local_id: self_param }))
    }
}

#[instrument(skip_all, fields(cmd_name))]
fn eval_let_binding(expr_json: &JsonObj, mut ctx: FnBodyBuilderCtx<'_>) -> Option<StatementKind> {
    trace!("Evaluating let binding json: {:?}", expr_json);
    assert_eq!(expr_json["type"].as_str().unwrap(), "let-binding");

    // create a new local variable with the given name
    let var_name = expr_json["varName"].as_str().unwrap();
    let local_id = ctx.mir.functions[ctx.fn_id].locals.insert(LocalDeclaration {
        debug_name: Some(var_name.to_string()),
        ty: MirType::Abstract(NetlogoAbstractType::Top),
        storage: LocalStorage::Register,
    });
    ctx.mir.fn_info[ctx.fn_id].local_names.insert(Rc::from(var_name), local_id);

    let value = expr_json["value"].as_object().unwrap();
    let value = eval_reporter(value, ctx.reborrow());

    let let_stmt = ctx.nodes.insert(Box::new(node::SetLocalVar { local_id, value }));
    Some(StatementKind::Node(let_stmt))
}

#[instrument(skip_all, fields(cmd_name))]
fn eval_command(expr_json: &JsonObj, mut ctx: FnBodyBuilderCtx<'_>) -> Option<StatementKind> {
    trace!("Evaluating command json: {:?}", expr_json);
    assert_eq!(expr_json["type"].as_str().unwrap(), "command-app");

    let args: Vec<&JsonObj> =
        expr_json["args"].as_array().unwrap().iter().map(|arg| arg.as_object().unwrap()).collect();

    let cmd_name = expr_json["name"].as_str().unwrap();
    Span::current().record("cmd_name", cmd_name);
    trace!("Handling command `{}`", cmd_name);
    match cmd_name {
        "STOP" => Some(StatementKind::Stop),
        "CLEAR-ALL" => {
            let context = ctx.get_context();
            let clear_all = ctx.nodes.insert(Box::new(node::ClearAll { context }));
            Some(StatementKind::Node(clear_all))
        }
        "SET-DEFAULT-SHAPE" => {
            // FUTURE implmement this instead of skipping it
            None
        }
        "CREATE-TURTLES" => {
            let &[population, body] = args.as_slice() else {
                panic!("expected two arguments for CREATE-TURTLES");
            };
            let context = ctx.get_context();
            let population = eval_reporter(population, ctx.reborrow());
            let body = eval_ephemeral_closure(body, ctx.fn_id, AgentClass::Turtle, ctx.reborrow());

            let create_turtles = ctx.nodes.insert(Box::new(node::CreateTurtles {
                context,
                breed: ctx.mir.global_names.turtle_breeds[&None],
                num_turtles: population,
                body,
            }));
            Some(StatementKind::Node(create_turtles))
        }
        "SET" => {
            let &[var, value] = args.as_slice() else {
                panic!("expected two arguments for SET");
            };

            let context = ctx.get_context();

            assert!(var["type"].as_str().unwrap() == "reporter-call");
            assert!(var["args"].as_array().unwrap().is_empty());
            let var_name = var["name"].as_str().unwrap();
            let var_desc = ctx.mir.global_names.lookup(var_name).unwrap();
            let value = eval_reporter(value, ctx.reborrow());

            // TODO this should also be able to work for local variables
            // the type of the variable being assigned determines which node to use
            match var_desc {
                NameReferent::TurtleVar(var_desc) => {
                    let turtle = ctx.get_self_turtle();
                    let set_turtle_var = ctx.nodes.insert(Box::new(node::SetTurtleVar {
                        context,
                        turtle,
                        var: var_desc,
                        value,
                    }));
                    Some(StatementKind::Node(set_turtle_var))
                }
                NameReferent::PatchVar(var_desc) => {
                    let agent = ctx.get_self_turtle_or_patch();
                    let set_patch_var =
                        ctx.nodes.insert(Box::new(node::SetPatchVarAsTurtleOrPatch {
                            context,
                            agent,
                            var: var_desc,
                            value,
                        }));
                    Some(StatementKind::Node(set_patch_var))
                }
                _ => todo!(),
            }
        }
        "FD" => {
            let &[distance] = args.as_slice() else {
                panic!("expected one argument for FD");
            };
            let context = ctx.get_context();
            let turtle = ctx.get_self_turtle();
            let distance = eval_reporter(distance, ctx.reborrow());
            let fd = ctx.nodes.insert(Box::new(node::TurtleForward { context, turtle, distance }));
            Some(StatementKind::Node(fd))
        }
        "LT" => {
            let &[heading] = args.as_slice() else {
                panic!("expected one argument for LT");
            };
            let context = ctx.get_context();
            let turtle = ctx.get_self_turtle();
            let angle_rt = eval_reporter(heading, ctx.reborrow());
            let angle_lt = ctx
                .nodes
                .insert(Box::new(node::UnaryOp { op: UnaryOpcode::Neg, operand: angle_rt }));
            let rt =
                ctx.nodes.insert(Box::new(node::TurtleRotate { context, turtle, angle: angle_lt }));
            Some(StatementKind::Node(rt))
        }
        "RT" => {
            let &[heading] = args.as_slice() else {
                panic!("expected one argument for RT");
            };
            let context = ctx.get_context();
            let turtle = ctx.get_self_turtle();
            let angle = eval_reporter(heading, ctx.reborrow());
            let rt = ctx.nodes.insert(Box::new(node::TurtleRotate { context, turtle, angle }));
            Some(StatementKind::Node(rt))
        }
        "RESET-TICKS" => {
            let &[] = args.as_slice() else {
                panic!("expected no arguments for RESET-TICKS");
            };
            let context = ctx.get_context();
            let reset_ticks = ctx.nodes.insert(Box::new(node::ResetTicks { context }));
            Some(StatementKind::Node(reset_ticks))
        }
        "ASK" => {
            let &[recipients, body] = args.as_slice() else {
                panic!("expected two arguments for ASK");
            };
            let context = ctx.get_context();
            let recipients = eval_reporter(recipients, ctx.reborrow());
            let body = eval_ephemeral_closure(body, ctx.fn_id, AgentClass::Any, ctx.reborrow());
            let ask = ctx.nodes.insert(Box::new(node::Ask { context, recipients, body }));
            Some(StatementKind::Node(ask))
        }
        "IF" | "IFELSE" => {
            // eagerly evaluate the condition
            let condition = eval_reporter(args[0], ctx.reborrow());

            // translate the inner statements
            let then_stmts = args[1]["statements"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|stmt_json| {
                    let stmt_json = stmt_json.as_object().unwrap();
                    eval_command(stmt_json, ctx.reborrow())
                })
                .collect();
            let else_stmts = if cmd_name == "IFELSE" {
                args[2]["statements"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|stmt_json| {
                        let stmt_json = stmt_json.as_object().unwrap();
                        eval_command(stmt_json, ctx.reborrow())
                    })
                    .collect()
            } else {
                Vec::new()
            };

            Some(StatementKind::IfElse {
                condition,
                then_block: StatementBlock { statements: then_stmts },
                else_block: StatementBlock { statements: else_stmts },
            })
        }
        "DIFFUSE" => {
            let &[var_name, amt] = args.as_slice() else {
                panic!("expected two arguments for DIFFUSE");
            };

            assert!(var_name["type"].as_str().unwrap() == "reporter-call");
            assert!(var_name["args"].as_array().unwrap().is_empty());
            let var_name = var_name["name"].as_str().unwrap();

            let Some(NameReferent::PatchVar(var_desc)) = ctx.mir.global_names.lookup(var_name)
            else {
                panic!("expected patch variable for DIFFUSE");
            };
            let context = ctx.get_context();
            let amt = eval_reporter(amt, ctx.reborrow());

            let diffuse =
                ctx.nodes.insert(Box::new(node::Diffuse { context, variable: var_desc, amt }));
            Some(StatementKind::Node(diffuse))
        }
        "TICK" => {
            let &[] = args.as_slice() else {
                panic!("expected no arguments for TICK");
            };
            let context = ctx.get_context();
            let tick = ctx.nodes.insert(Box::new(node::AdvanceTick { context }));
            Some(StatementKind::Node(tick))
        }
        "REPORT" => {
            let &[expr] = args.as_slice() else {
                panic!("expected one argument for REPORT");
            };
            let value = eval_reporter(expr, ctx.reborrow());
            Some(StatementKind::Return { value })
        }
        cmd_name => {
            // at this point, assume that the command is a user-defined
            // procedure call
            let target_fn = ctx.mir.global_names.lookup(cmd_name).unwrap();
            if let NameReferent::UserProc(fn_id) = target_fn {
                // eagerly evaluate all arguments
                let evaled_args =
                    args.iter().map(|&arg| eval_reporter(arg, ctx.reborrow())).collect::<Vec<_>>();

                let call_user_fn = ctx
                    .nodes
                    .insert(Box::new(node::CallUserFn { target: fn_id, args: evaled_args }));
                Some(StatementKind::Node(call_user_fn))
            } else {
                unimplemented!();
            }
        }
    }
}

#[instrument(skip_all, fields(name))]
fn eval_reporter(expr_json: &JsonObj, mut ctx: FnBodyBuilderCtx<'_>) -> NodeId {
    match expr_json["type"].as_str().unwrap() {
        "reporter-call" | "reporter-proc-call" => {
            let reporter_name = expr_json["name"].as_str().unwrap();
            Span::current().record("name", reporter_name);
            trace!("Handling reporter call `{}`", reporter_name);

            let args: Vec<&JsonObj> = expr_json["args"]
                .as_array()
                .unwrap()
                .iter()
                .map(|arg| arg.as_object().unwrap())
                .collect();

            match reporter_name {
                "OF" => {
                    let &[body, recipients] = args.as_slice() else {
                        panic!("expected two arguments for OF");
                    };
                    let context = ctx.get_context();
                    let recipients = eval_reporter(recipients, ctx.reborrow());
                    let body =
                        eval_ephemeral_closure(body, ctx.fn_id, AgentClass::Any, ctx.reborrow());
                    ctx.nodes.insert(Box::new(node::Of { context, recipients, body }))
                }
                "<" | ">" | "=" | "<=" | ">=" | "-" | "+" | "*" | "/" | "AND" | "OR" => {
                    let &[lhs, rhs] = args.as_slice() else {
                        panic!("expected two arguments for `{}`", reporter_name);
                    };
                    let lhs = eval_reporter(lhs, ctx.reborrow());
                    let rhs = eval_reporter(rhs, ctx.reborrow());
                    let op = match reporter_name {
                        "<" => BinaryOpcode::Lt,
                        ">" => BinaryOpcode::Gt,
                        "=" => BinaryOpcode::Eq,
                        "<=" => BinaryOpcode::Lte,
                        ">=" => BinaryOpcode::Gte,
                        "-" => BinaryOpcode::Sub,
                        "+" => BinaryOpcode::Add,
                        "*" => BinaryOpcode::Mul,
                        "/" => BinaryOpcode::Div,
                        "AND" => BinaryOpcode::And,
                        "OR" => BinaryOpcode::Or,
                        _ => unreachable!(),
                    };
                    let less_than =
                        ctx.nodes.insert(Box::new(node::BinaryOperation { op, lhs, rhs }));
                    less_than
                }
                "NOT" => {
                    let &[operand] = args.as_slice() else {
                        panic!("expected one argument for `NOT`");
                    };
                    let operand = eval_reporter(operand, ctx.reborrow());
                    ctx.nodes.insert(Box::new(node::UnaryOp { op: UnaryOpcode::Not, operand }))
                }
                "DISTANCEXY" => {
                    let &[x, y] = args.as_slice() else {
                        panic!("expected two arguments for `DISTANCEXY`");
                    };
                    let agent = ctx.get_self_turtle_or_patch();
                    let x = eval_reporter(x, ctx.reborrow());
                    let y = eval_reporter(y, ctx.reborrow());
                    ctx.nodes.insert(Box::new(node::Distancexy { agent, x, y }))
                }
                "CAN-MOVE?" => {
                    let &[distance] = args.as_slice() else {
                        panic!("expected one argument for `CAN-MOVE?`");
                    };
                    let turtle = ctx.get_self_turtle();
                    let distance = eval_reporter(distance, ctx.reborrow());
                    ctx.nodes.insert(Box::new(node::CanMove { turtle, distance }))
                }
                reporter_name @ ("PATCH-RIGHT-AND-AHEAD" | "PATCH-LEFT-AND-AHEAD") => {
                    let &[heading, distance] = args.as_slice() else {
                        panic!(
                            "expected two arguments for `PATCH-RIGHT-AND-AHEAD` or `PATCH-LEFT-AND-AHEAD`"
                        );
                    };
                    let turtle = ctx.get_self_turtle();
                    let relative_loc = match reporter_name {
                        "PATCH-RIGHT-AND-AHEAD" => PatchLocRelation::RightAhead,
                        "PATCH-LEFT-AND-AHEAD" => PatchLocRelation::LeftAhead,
                        _ => unreachable!(),
                    };
                    let heading = eval_reporter(heading, ctx.reborrow());
                    let distance = eval_reporter(distance, ctx.reborrow());
                    ctx.nodes.insert(Box::new(node::PatchRelative {
                        relative_loc,
                        turtle,
                        distance,
                        heading,
                    }))
                }
                "MAX-PXCOR" => {
                    let &[] = args.as_slice() else {
                        panic!("expected no arguments for `MAX-PXCOR`");
                    };
                    let context = ctx.get_context();
                    ctx.nodes.insert(Box::new(node::MaxPxcor { context }))
                }
                "MAX-PYCOR" => {
                    let &[] = args.as_slice() else {
                        panic!("expected no arguments for `MAX-PYCOR`");
                    };
                    let context = ctx.get_context();
                    ctx.nodes.insert(Box::new(node::MaxPycor { context }))
                }
                "ONE-OF" => {
                    // TODO actually implement this
                    ctx.nodes.insert(Box::new(node::Constant { value: UnpackedDynBox::Float(1.0) }))
                }
                "SCALE-COLOR" => {
                    let &[color, number, range1, range2] = args.as_slice() else {
                        panic!("expected four arguments for `SCALE-COLOR`");
                    };
                    let color = eval_reporter(color, ctx.reborrow());
                    let number = eval_reporter(number, ctx.reborrow());
                    let range1 = eval_reporter(range1, ctx.reborrow());
                    let range2 = eval_reporter(range2, ctx.reborrow());
                    ctx.nodes.insert(Box::new(node::ScaleColor { color, number, range1, range2 }))
                }
                "TICKS" => {
                    let &[] = args.as_slice() else {
                        panic!("expected no arguments for `TICKS`");
                    };
                    let context = ctx.get_context();
                    ctx.nodes.insert(Box::new(node::GetTick { context }))
                }
                "RANDOM" => {
                    let &[bound] = args.as_slice() else {
                        panic!("expected one argument for `RANDOM`");
                    };
                    let bound = eval_reporter(bound, ctx.reborrow());
                    ctx.nodes.insert(Box::new(node::RandomInt { bound }))
                }
                other if let Some(val) = ctx.mir.global_names.lookup(other) => match val {
                    NameReferent::Constant(mk_node) => ctx.nodes.insert(mk_node()),
                    NameReferent::PatchVar(var_desc) => {
                        let context = ctx.get_context();
                        let agent = ctx.get_self_turtle_or_patch();
                        ctx.nodes.insert(Box::new(node::GetPatchVarAsTurtleOrPatch {
                            context,
                            agent,
                            var: var_desc,
                        }))
                    }
                    NameReferent::TurtleVar(var_desc) => {
                        let context = ctx.get_context();
                        let turtle = ctx.get_self_turtle();
                        ctx.nodes.insert(Box::new(node::GetTurtleVar {
                            context,
                            turtle,
                            var: var_desc,
                        }))
                    }
                    NameReferent::UserProc(fn_id) => {
                        // eagerly evaluate all arguments
                        let evaled_args = args
                            .iter()
                            .map(|&arg| eval_reporter(arg, ctx.reborrow()))
                            .collect::<Vec<_>>();

                        ctx.nodes
                            .insert(Box::new(node::CallUserFn { target: fn_id, args: evaled_args }))
                    }
                    referent => {
                        error!("reporter has referent: {:?}", referent);
                        todo!("reporter has referent: {:?}", referent);
                    }
                },
                other if let Some(local_id) = ctx.mir.fn_info[ctx.fn_id].local_names.get(other) => {
                    ctx.nodes.insert(Box::new(node::GetLocalVar { local_id: *local_id }))
                }
                _ => unreachable!("unknown reporter: {}", reporter_name),
            }
        }
        "string" => todo!(),
        "number" => {
            let val = expr_json["value"].as_f64().unwrap();
            ctx.nodes.insert(Box::new(node::Constant { value: UnpackedDynBox::Float(val) }))
        }
        "nobody" => ctx.nodes.insert(Box::new(node::Constant { value: UnpackedDynBox::Nobody })),
        r#type => {
            error!("unknown reporter type: {}", r#type);
            panic!();
        }
    }
}

#[instrument(skip_all)]
fn eval_ephemeral_closure(
    expr_json: &JsonObj,
    parent_fn_id: FunctionId,
    agent_class: AgentClass,
    ctx: FnBodyBuilderCtx<'_>,
) -> NodeId {
    trace!("Evaluating ephemeral closure");

    enum ClosureType {
        CommandBlock,
        ReporterBlock,
    }

    let closure_type = match expr_json["type"].as_str().unwrap() {
        "command-block" => ClosureType::CommandBlock,
        "reporter-block" => ClosureType::ReporterBlock,
        _ => unimplemented!(),
    };

    // generate a proc name
    let parent_debug_name = ctx.mir.fn_info[parent_fn_id].debug_name.as_ref();
    let proc_name = format!("{} body", parent_debug_name);

    // calculate the function parameters
    let mut this_fn_info = FnInfo::new(Rc::from(proc_name.as_str()));
    let mut locals = SlotMap::with_key();
    let mut parameter_locals = Vec::new();
    // add the environment pointer
    let env_param = locals.insert(LocalDeclaration {
        debug_name: Some("env".to_string()),
        ty: MirType::Machine(NetlogoMachineType::UNTYPED_PTR),
        storage: LocalStorage::Register,
    });
    parameter_locals.push(env_param);
    this_fn_info.env_param = Some(env_param);
    // add the context parameter
    let context_param = locals.insert(LocalDeclaration {
        debug_name: Some("context".to_string()),
        ty: MirType::Machine(NetlogoMachineType::UNTYPED_PTR),
        storage: LocalStorage::Register,
    });
    parameter_locals.push(context_param);
    this_fn_info.context_param = Some(context_param);
    // add the self parameter
    match agent_class {
        AgentClass::Observer => {
            trace!("No self parameter needed for Observer agent class");
        }
        AgentClass::Turtle => {
            let local_id = locals.insert(LocalDeclaration {
                debug_name: Some("self_turtle_id".to_string()),
                ty: MirType::Machine(NetlogoMachineType::TURTLE_ID),
                storage: LocalStorage::Register,
            });
            parameter_locals.push(local_id);
            this_fn_info.self_param = Some((AgentClass::Turtle, local_id));
            trace!("Added turtle self parameter with local_id: {:?}", local_id);
        }
        AgentClass::Patch => {
            let local_id = locals.insert(LocalDeclaration {
                debug_name: Some("self_patch_id".to_string()),
                ty: MirType::Machine(NetlogoMachineType::PATCH_ID),
                storage: LocalStorage::Register,
            });
            parameter_locals.push(local_id);
            this_fn_info.self_param = Some((AgentClass::Patch, local_id));
            trace!("Added patch self parameter with local_id: {:?}", local_id);
        }
        AgentClass::Link => todo!(),
        AgentClass::Any => {
            let local_id = locals.insert(LocalDeclaration {
                debug_name: Some("self_any".to_string()),
                ty: MirType::Abstract(NetlogoAbstractType::Top),
                storage: LocalStorage::Register,
            });
            parameter_locals.push(local_id);
            this_fn_info.self_param = Some((AgentClass::Any, local_id));
            trace!("Added any self parameter with local_id: {:?}", local_id);
        }
    }

    let return_ty = match closure_type {
        ClosureType::CommandBlock => NetlogoAbstractType::Unit,
        ClosureType::ReporterBlock => NetlogoAbstractType::Top,
    };

    // create the function skeleton
    let function = Function {
        debug_name: Some(proc_name),
        parameters: parameter_locals,
        return_ty,
        locals,
        // cfg and nodes are defaulted and will be filled in after
        cfg: StatementBlock { statements: vec![] },
        nodes: RefCell::new(SlotMap::with_key()),
    };
    let fn_id = ctx.mir.functions.insert(function);
    ctx.mir.fn_info.insert(fn_id, this_fn_info);
    trace!("Inserted function for closure with id: {:?}", fn_id);
    trace!("Current function ids: {:?}", ctx.mir.functions.keys().collect::<Vec<_>>());

    // build the function body
    let statements = match closure_type {
        ClosureType::CommandBlock => expr_json["statements"].as_array().unwrap(),
        ClosureType::ReporterBlock => {
            let cmd = json!(
                {
                    "type": "command-app",
                    "name": "REPORT",
                    "args": [
                        expr_json["reporterApp"]
                    ]
                }
            );
            &vec![cmd]
        }
    };
    build_body(statements, fn_id, ctx.mir).unwrap();

    // return a closure object
    let closure = ctx.nodes.insert(Box::new(node::Closure {
        captures: vec![], // TODO add captures
        body: fn_id,
    }));
    closure
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ants() {
        tracing_subscriber::fmt().with_max_level(tracing::Level::TRACE).init();

        let json = include_str!("../../bench/models/ants/ast.json");
        let json: JsonValue = serde_json::from_str(json).unwrap();
        let mir = ast_to_mir(&json).unwrap();

        let debug_output = format!("{:#?}", mir);
        std::fs::write("mir_debug.txt", debug_output).expect("Failed to write MIR debug output");

        for (fn_id, function) in mir.functions {
            let function = function.borrow();
            let dot_string = function.to_dot_string_with_options(false);
            let filename =
                format!("{}-{:?}.dot", fn_id, function.debug_name.as_deref().unwrap_or("unnamed"));
            trace!("Writing DOT file for function {:?}: {}", fn_id, filename);
            std::fs::write(filename, dot_string).expect("Failed to write DOT file");
        }
    }
}
