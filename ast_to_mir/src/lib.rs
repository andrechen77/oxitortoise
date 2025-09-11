#![feature(if_let_guard)]

use std::{collections::HashMap, rc::Rc};

use engine::{
    mir::{
        self, EffectfulNode, Function, FunctionId, LocalDeclaration, LocalId, LocalStorage, NodeId,
        StatementBlock, StatementKind,
        node::{self, BinaryOpcode},
    },
    sim::{
        patch::PatchVarDesc,
        turtle::{BreedId, TurtleVarDesc},
        value::{NetlogoInternalType, UnpackedDynBox},
    },
    slotmap::{SecondaryMap, SlotMap, new_key_type},
};
use serde_json::Value as JsonValue;
use tracing::{Span, error, instrument, trace};

type JsonObj = serde_json::Map<String, JsonValue>;

// TODO this should be a part of MIR
new_key_type! {
    pub struct GlobalId;
}

// TODO this should work with local variables too
#[derive(Debug)]
struct NameScope {
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

impl NameScope {
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
    name_scope: NameScope,
}

pub fn ast_to_mir(ast: &JsonValue) -> anyhow::Result<mir::Program> {
    trace!("Starting AST to MIR conversion");
    let root = ast.as_object().unwrap();

    let mut globals: SlotMap<GlobalId, ()> = SlotMap::with_key();

    let mut turtle_breeds: SlotMap<BreedId, ()> = SlotMap::with_key();
    let default_turtle_breed = turtle_breeds.insert(());

    let mut name_scope = NameScope::with_builtins(default_turtle_breed);
    trace!("Initialized name scope with builtins");

    let meta_vars = root["metaVars"].as_object().unwrap();
    // add global variables
    for global in meta_vars["globals"].as_array().unwrap() {
        let global_name = global.as_str().unwrap();
        let global_id = globals.insert(());
        name_scope.globals.insert(Rc::from(global_name), global_id);
        trace!("Added global variable `{}` with id {:?}", global_name, global_id);
    }
    // add patch variables
    for (i, patch_var) in meta_vars["patchVars"].as_array().unwrap().iter().enumerate() {
        let patch_var_name = patch_var.as_str().unwrap();
        let patch_var_id = PatchVarDesc::Custom(i);
        name_scope.patch_vars.insert(Rc::from(patch_var_name), patch_var_id);
        trace!("Added patch variable `{}` with id {:?}", patch_var_name, patch_var_id);
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

    let mut mir_builder = MirBuilder { globals, turtle_breeds, functions, fn_info, name_scope };

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

    trace!("end of procedure so far. {:?}", mir_builder);
    todo!()
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
        ty: NetlogoInternalType::UNTYPED_PTR,
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
                ty: NetlogoInternalType::TURTLE_ID,
                storage: LocalStorage::Register,
            });
            parameter_locals.push(local_id);
            fn_info.self_param = Some((AgentClass::Turtle, local_id));
            trace!("Added turtle self parameter with local_id: {:?}", local_id);
        }
        AgentClass::Patch => {
            let local_id = locals.insert(LocalDeclaration {
                debug_name: Some("self_patch_id".to_string()),
                ty: NetlogoInternalType::PATCH_ID,
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
            ty: NetlogoInternalType::DYN_BOX,
            storage: LocalStorage::Register,
        });
        parameter_locals.push(local_id);
        fn_info.positional_params.push(local_id);
        trace!("Added positional parameter {} with local_id: {:?}", arg, local_id);
    }

    // calculate the function return type
    let return_ty = match procedure["returnType"].as_str().unwrap() {
        "unit" => None,
        "wildcard" => Some(NetlogoInternalType::DYN_BOX),
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
        nodes: SlotMap::with_key(),
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

    // The statements of the current control flow construct
    let statements: Vec<StatementKind> = statements_json
        .iter()
        .filter_map(|stmt_json| {
            let statement_json = stmt_json.as_object().unwrap();
            eval_command(statement_json, fn_id, mir_builder, &mut nodes)
        })
        .collect();

    trace!("finished building body");
    Ok(())
}

#[instrument(skip_all, fields(cmd_name))]
fn eval_command(
    expr_json: &JsonObj,
    containing_fn: FunctionId,
    mir_builder: &mut MirBuilder,
    nodes: &mut SlotMap<NodeId, Box<dyn EffectfulNode>>,
    // locals: &mut SlotMap<LocalId, LocalDeclaration>,
) -> Option<StatementKind> {
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
            let context = nodes.insert(Box::new(node::GetLocalVar {
                local_id: mir_builder.fn_info[containing_fn].context_param.unwrap(),
            }));
            let clear_all = nodes.insert(Box::new(node::ClearAll { context }));
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
            let context = nodes.insert(Box::new(node::GetLocalVar {
                local_id: mir_builder.fn_info[containing_fn].context_param.unwrap(),
            }));

            let population = eval_reporter(population, containing_fn, mir_builder, nodes);
            let body =
                eval_ephemeral_closure(body, containing_fn, AgentClass::Turtle, mir_builder, nodes);

            let create_turtles = nodes.insert(Box::new(node::CreateTurtles {
                context,
                breed: mir_builder.name_scope.turtle_breeds[&None],
                num_turtles: population,
                body,
            }));
            Some(StatementKind::Node(create_turtles))
        }
        // "LET" => {
        //     let &[var_name, value] = args.as_slice() else {
        //         panic!("expected two arguments for LET");
        //     };

        //     // create a new local variable with the given name
        //     let var_name = var_name["reporter"].as_object().unwrap()["name"].as_str().unwrap();
        //     let local_id = locals.insert(LocalDeclaration {
        //         debug_name: Some(var_name.to_string()),
        //         ty: NetlogoInternalType::DYN_BOX,
        //         storage: LocalStorage::Register,
        //     });

        //     let value = eval_reporter(value, containing_fn, mir_builder, nodes);

        //     let let_stmt = nodes.insert(Box::new(node::SetLocalVar { local_id, value }));
        //     Some(StatementKind::Node(let_stmt))
        // }
        "SET" => {
            let &[var, value] = args.as_slice() else {
                panic!("expected two arguments for SET");
            };

            let context = nodes.insert(Box::new(node::GetLocalVar {
                local_id: mir_builder.fn_info[containing_fn].context_param.unwrap(),
            }));

            assert!(var["type"].as_str().unwrap() == "reporter-call");
            assert!(var["args"].as_array().unwrap().is_empty());
            let var_name = var["name"].as_str().unwrap();
            let var_desc = mir_builder.name_scope.lookup(var_name).unwrap();
            let value = eval_reporter(value, containing_fn, mir_builder, nodes);

            // TODO this should also be able to work for local variables
            // the type of the variable being assigned determines which node to use
            match var_desc {
                NameReferent::TurtleVar(var_desc) => {
                    let (agent_class, self_param) =
                        mir_builder.fn_info[containing_fn].self_param.unwrap();
                    assert!(agent_class.is_turtle());
                    let turtle_id =
                        nodes.insert(Box::new(node::GetLocalVar { local_id: self_param }));
                    let set_turtle_var = nodes.insert(Box::new(node::SetTurtleVar {
                        context,
                        turtle: turtle_id,
                        var: var_desc,
                        value,
                    }));
                    Some(StatementKind::Node(set_turtle_var))
                }
                NameReferent::PatchVar(var_desc) => {
                    let (agent_class, self_param) =
                        mir_builder.fn_info[containing_fn].self_param.unwrap();
                    assert!(agent_class.is_patch() || agent_class.is_turtle());
                    let agent_id =
                        nodes.insert(Box::new(node::GetLocalVar { local_id: self_param }));
                    let set_patch_var = nodes.insert(Box::new(node::SetPatchVarAsTurtleOrPatch {
                        context,
                        agent: agent_id,
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
            let context = nodes.insert(Box::new(node::GetLocalVar {
                local_id: mir_builder.fn_info[containing_fn].context_param.unwrap(),
            }));
            let (agent_class, self_param) = mir_builder.fn_info[containing_fn].self_param.unwrap();
            assert!(agent_class.is_turtle());
            let turtle = nodes.insert(Box::new(node::GetLocalVar { local_id: self_param }));
            let distance = eval_reporter(distance, containing_fn, mir_builder, nodes);
            let fd = nodes.insert(Box::new(node::TurtleForward { context, turtle, distance }));
            Some(StatementKind::Node(fd))
        }
        "RT" => {
            let &[heading] = args.as_slice() else {
                panic!("expected one argument for RT");
            };
            let context = nodes.insert(Box::new(node::GetLocalVar {
                local_id: mir_builder.fn_info[containing_fn].context_param.unwrap(),
            }));
            let (agent_class, self_param) = mir_builder.fn_info[containing_fn].self_param.unwrap();
            assert!(agent_class.is_turtle());
            let turtle = nodes.insert(Box::new(node::GetLocalVar { local_id: self_param }));
            let angle = eval_reporter(heading, containing_fn, mir_builder, nodes);
            let rt = nodes.insert(Box::new(node::TurtleRotate { context, turtle, angle }));
            Some(StatementKind::Node(rt))
        }
        "RESET-TICKS" => {
            let &[] = args.as_slice() else {
                panic!("expected no arguments for RESET-TICKS");
            };
            let context = nodes.insert(Box::new(node::GetLocalVar {
                local_id: mir_builder.fn_info[containing_fn].context_param.unwrap(),
            }));
            let reset_ticks = nodes.insert(Box::new(node::ResetTicks { context }));
            Some(StatementKind::Node(reset_ticks))
        }
        "ASK" => {
            let &[recipients, body] = args.as_slice() else {
                panic!("expected two arguments for ASK");
            };
            let context = nodes.insert(Box::new(node::GetLocalVar {
                local_id: mir_builder.fn_info[containing_fn].context_param.unwrap(),
            }));
            let recipients = eval_reporter(recipients, containing_fn, mir_builder, nodes);
            let body =
                eval_ephemeral_closure(body, containing_fn, AgentClass::Any, mir_builder, nodes);
            let ask = nodes.insert(Box::new(node::Ask { context, recipients, body }));
            Some(StatementKind::Node(ask))
        }
        "IF" | "IFELSE" => {
            // eagerly evaluate the condition
            let condition = eval_reporter(args[0], containing_fn, mir_builder, nodes);

            // translate the inner statements
            let then_stmts = args[1]["statements"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|stmt_json| {
                    let stmt_json = stmt_json.as_object().unwrap();
                    eval_command(stmt_json, containing_fn, mir_builder, nodes)
                })
                .collect();
            let else_stmts = if cmd_name == "IFELSE" {
                args[2]["statements"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|stmt_json| {
                        let stmt_json = stmt_json.as_object().unwrap();
                        eval_command(stmt_json, containing_fn, mir_builder, nodes)
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

            let Some(NameReferent::PatchVar(var_desc)) = mir_builder.name_scope.lookup(var_name)
            else {
                panic!("expected patch variable for DIFFUSE");
            };
            let context = nodes.insert(Box::new(node::GetLocalVar {
                local_id: mir_builder.fn_info[containing_fn].context_param.unwrap(),
            }));
            let amt = eval_reporter(amt, containing_fn, mir_builder, nodes);

            let diffuse =
                nodes.insert(Box::new(node::Diffuse { context, variable: var_desc, amt }));
            Some(StatementKind::Node(diffuse))
        }
        "TICK" => {
            let &[] = args.as_slice() else {
                panic!("expected no arguments for TICK");
            };
            let context = nodes.insert(Box::new(node::GetLocalVar {
                local_id: mir_builder.fn_info[containing_fn].context_param.unwrap(),
            }));
            let tick = nodes.insert(Box::new(node::AdvanceTick { context }));
            Some(StatementKind::Node(tick))
        }
        cmd_name => {
            // at this point, assume that the command is a user-defined
            // procedure call
            let target_fn = mir_builder.name_scope.lookup(cmd_name).unwrap();
            if let NameReferent::UserProc(fn_id) = target_fn {
                // eagerly evaluate all arguments
                let evaled_args = args
                    .iter()
                    .map(|&arg| eval_reporter(arg, containing_fn, mir_builder, nodes))
                    .collect::<Vec<_>>();

                let call_user_fn =
                    nodes.insert(Box::new(node::CallUserFn { target: fn_id, args: evaled_args }));
                Some(StatementKind::Node(call_user_fn))
            } else {
                unimplemented!();
            }
        }
    }
}

#[instrument(skip_all, fields(name))]
fn eval_reporter(
    expr_json: &JsonObj,
    containing_fn: FunctionId,
    mir_builder: &mut MirBuilder,
    nodes: &mut SlotMap<NodeId, Box<dyn EffectfulNode>>,
) -> NodeId {
    match expr_json["type"].as_str().unwrap() {
        "reporter-call" => {
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
                "<" | ">" | "=" | "<=" | ">=" | "-" | "+" | "*" | "/" | "AND" | "OR" => {
                    let &[lhs, rhs] = args.as_slice() else {
                        panic!("expected two arguments for `{}`", reporter_name);
                    };
                    let lhs = eval_reporter(lhs, containing_fn, mir_builder, nodes);
                    let rhs = eval_reporter(rhs, containing_fn, mir_builder, nodes);
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
                    let less_than = nodes.insert(Box::new(node::BinaryOperation { op, lhs, rhs }));
                    less_than
                }
                "DISTANCEXY" => {
                    let &[x, y] = args.as_slice() else {
                        panic!("expected two arguments for `DISTANCEXY`");
                    };
                    let (agent_class, self_param) =
                        mir_builder.fn_info[containing_fn].self_param.unwrap();
                    assert!(agent_class.is_turtle() || agent_class.is_patch());
                    let agent = nodes.insert(Box::new(node::GetLocalVar { local_id: self_param }));
                    let x = eval_reporter(x, containing_fn, mir_builder, nodes);
                    let y = eval_reporter(y, containing_fn, mir_builder, nodes);
                    nodes.insert(Box::new(node::Distancexy { agent, x, y }))
                }
                "MAX-PXCOR" => {
                    let &[] = args.as_slice() else {
                        panic!("expected no arguments for `MAX-PXCOR`");
                    };
                    let context = nodes.insert(Box::new(node::GetLocalVar {
                        local_id: mir_builder.fn_info[containing_fn].context_param.unwrap(),
                    }));
                    nodes.insert(Box::new(node::MaxPxcor { context }))
                }
                "MAX-PYCOR" => {
                    let &[] = args.as_slice() else {
                        panic!("expected no arguments for `MAX-PYCOR`");
                    };
                    let context = nodes.insert(Box::new(node::GetLocalVar {
                        local_id: mir_builder.fn_info[containing_fn].context_param.unwrap(),
                    }));
                    nodes.insert(Box::new(node::MaxPycor { context }))
                }
                "ONE-OF" => {
                    // TODO actually implement this
                    nodes.insert(Box::new(node::Constant { value: UnpackedDynBox::Float(1.0) }))
                }
                "SCALE-COLOR" => {
                    let &[color, number, range1, range2] = args.as_slice() else {
                        panic!("expected four arguments for `SCALE-COLOR`");
                    };
                    let color = eval_reporter(color, containing_fn, mir_builder, nodes);
                    let number = eval_reporter(number, containing_fn, mir_builder, nodes);
                    let range1 = eval_reporter(range1, containing_fn, mir_builder, nodes);
                    let range2 = eval_reporter(range2, containing_fn, mir_builder, nodes);
                    nodes.insert(Box::new(node::ScaleColor { color, number, range1, range2 }))
                }
                "TICKS" => {
                    let &[] = args.as_slice() else {
                        panic!("expected no arguments for `TICKS`");
                    };
                    let context = nodes.insert(Box::new(node::GetLocalVar {
                        local_id: mir_builder.fn_info[containing_fn].context_param.unwrap(),
                    }));
                    nodes.insert(Box::new(node::GetTick { context }))
                }
                other if let Some(val) = mir_builder.name_scope.lookup(other) => match val {
                    NameReferent::Constant(mk_node) => nodes.insert(mk_node()),
                    NameReferent::PatchVar(var_desc) => {
                        let context = nodes.insert(Box::new(node::GetLocalVar {
                            local_id: mir_builder.fn_info[containing_fn].context_param.unwrap(),
                        }));
                        let (agent_class, self_param) =
                            mir_builder.fn_info[containing_fn].self_param.unwrap();
                        assert!(agent_class.is_patch() || agent_class.is_turtle());
                        let self_id =
                            nodes.insert(Box::new(node::GetLocalVar { local_id: self_param }));
                        nodes.insert(Box::new(node::GetPatchVarAsTurtleOrPatch {
                            context,
                            agent: self_id,
                            var: var_desc,
                        }))
                    }
                    NameReferent::TurtleVar(var_desc) => {
                        let context = nodes.insert(Box::new(node::GetLocalVar {
                            local_id: mir_builder.fn_info[containing_fn].context_param.unwrap(),
                        }));
                        let (agent_class, self_param) =
                            mir_builder.fn_info[containing_fn].self_param.unwrap();
                        assert!(agent_class.is_turtle());
                        let self_id =
                            nodes.insert(Box::new(node::GetLocalVar { local_id: self_param }));
                        nodes.insert(Box::new(node::GetTurtleVar {
                            context,
                            turtle: self_id,
                            var: var_desc,
                        }))
                    }
                    referent => {
                        error!("reporter has referent: {:?}", referent);
                        todo!("reporter has referent: {:?}", referent);
                    }
                },
                _ => unreachable!("unknown reporter: {}", reporter_name),
            }
        }
        "string" => todo!(),
        "number" => {
            let val = expr_json["value"].as_f64().unwrap();
            nodes.insert(Box::new(node::Constant { value: UnpackedDynBox::Float(val) }))
        }
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
    mir_builder: &mut MirBuilder,
    nodes: &mut SlotMap<NodeId, Box<dyn EffectfulNode>>,
) -> NodeId {
    trace!("Evaluating ephemeral closure");
    assert_eq!(expr_json["type"].as_str().unwrap(), "command-block");

    // generate a proc name
    let parent_debug_name = mir_builder.fn_info[parent_fn_id].debug_name.as_ref();
    let proc_name = format!("{}/body", parent_debug_name);

    // calculate the function parameters
    let mut this_fn_info = FnInfo::new(Rc::from(proc_name.as_str()));
    let mut locals = SlotMap::with_key();
    let mut parameter_locals = Vec::new();
    // add the environment pointer
    let env_param = locals.insert(LocalDeclaration {
        debug_name: Some("env".to_string()),
        ty: NetlogoInternalType::UNTYPED_PTR,
        storage: LocalStorage::Register,
    });
    parameter_locals.push(env_param);
    this_fn_info.env_param = Some(env_param);
    // add the context parameter
    let context_param = locals.insert(LocalDeclaration {
        debug_name: Some("context".to_string()),
        ty: NetlogoInternalType::UNTYPED_PTR,
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
                ty: NetlogoInternalType::TURTLE_ID,
                storage: LocalStorage::Register,
            });
            parameter_locals.push(local_id);
            this_fn_info.self_param = Some((AgentClass::Turtle, local_id));
            trace!("Added turtle self parameter with local_id: {:?}", local_id);
        }
        AgentClass::Patch => {
            let local_id = locals.insert(LocalDeclaration {
                debug_name: Some("self_patch_id".to_string()),
                ty: NetlogoInternalType::PATCH_ID,
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
                ty: NetlogoInternalType::DYN_BOX,
                storage: LocalStorage::Register,
            });
            parameter_locals.push(local_id);
            this_fn_info.self_param = Some((AgentClass::Any, local_id));
            trace!("Added any self parameter with local_id: {:?}", local_id);
        }
    }

    // create the function skeleton
    let function = Function {
        debug_name: Some(proc_name),
        parameters: parameter_locals,
        return_ty: None,
        locals,
        // cfg and nodes are defaulted and will be filled in after
        cfg: StatementBlock { statements: vec![] },
        nodes: SlotMap::with_key(),
    };
    let fn_id = mir_builder.functions.insert(function);
    mir_builder.fn_info.insert(fn_id, this_fn_info);

    // build the function body
    build_body(expr_json["statements"].as_array().unwrap(), fn_id, mir_builder).unwrap();

    // return a closure object
    let closure = nodes.insert(Box::new(node::Closure {
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
        let mir = ast_to_mir(&json);
    }
}
