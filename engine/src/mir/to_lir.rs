use std::{collections::HashMap, mem::offset_of};

use lir::{
    Const,
    smallvec::{SmallVec, smallvec},
    typed_index_collections::TiVec,
};
use slotmap::{Key as _, SlotMap};

use crate::{
    exec::CanonExecutionContext,
    mir::{
        AgentField, Constant, DirectOperator, Function, FunctionId, ImmClosure, LocalId, Operand,
        Operation, Operator, Program, StatementBlock, StatementKind,
    },
    sim::{
        agent_schema::AgentFieldDescriptor,
        turtle::{OFFSET_TURTLES_TO_DATA, TurtleBaseData},
        value::{NetlogoInternalType, UnpackedDynBox},
        world::World,
    },
    util::row_buffer::RowBuffer,
    workspace::Workspace,
};

pub struct LirProgramBuilder {
    /// The LIR program being built.
    product: lir::Program,
    user_function_tracker: SlotMap<lir::FunctionId, ()>,
    available_host_functions: HashMap<&'static str, lir::HostFunctionId>,
    /// Maps each MIR function to an LIR function.
    available_user_functions: HashMap<FunctionId, lir::FunctionId>,
    /// The LIR signatures of each MIR function.
    function_signatures: HashMap<lir::FunctionId, (Vec<lir::ValType>, SmallVec<[lir::ValType; 1]>)>,
}

impl LirProgramBuilder {
    pub fn new(workspace: &Workspace, mir: &Program) -> Self {
        let mut builder = Self {
            product: lir::Program::default(),
            user_function_tracker: SlotMap::with_key(),
            available_host_functions: HashMap::new(),
            available_user_functions: HashMap::new(),
            function_signatures: HashMap::new(),
        };

        // translate at least the function signatures
        for (function_id, function) in mir.functions.iter() {
            let signature = translate_function_signature(function);
            // allocate a new function id for the LIR function
            let lir_fn_id = builder.user_function_tracker.insert(());
            builder.available_user_functions.insert(function_id, lir_fn_id);
            builder.function_signatures.insert(lir_fn_id, signature);
        }

        // add some host functions
        // TODO these should be initialized elsewhere
        let clear_all = builder.product.host_functions.push_and_get_key(lir::HostFunction {
            name: "clear_all",
            parameter_types: vec![lir::ValType::Ptr],
            return_type: vec![],
        });
        builder.available_host_functions.insert("clear_all", clear_all);

        let create_turtles = builder.product.host_functions.push_and_get_key(lir::HostFunction {
            name: "create_turtles",
            parameter_types: vec![
                lir::ValType::Ptr,
                lir::ValType::I32,
                lir::ValType::I32,
                lir::ValType::Ptr,
                lir::ValType::Ptr,
            ],
            return_type: vec![],
        });
        builder.available_host_functions.insert("create_turtles", create_turtles);

        builder
    }
}

fn translate_function_signature(
    function: &Function,
) -> (Vec<lir::ValType>, SmallVec<[lir::ValType; 1]>) {
    let mut params = Vec::new();
    if function.takes_env {
        params.push(lir::ValType::Ptr);
    }
    if function.takes_context {
        params.push(lir::ValType::Ptr);
    }
    for parameter in &function.parameters {
        params.extend(function.locals[*parameter].ty.to_lir_type());
    }
    let return_value = function.return_value.map(|l| function.locals[l].ty.to_lir_type());
    (params, return_value.unwrap_or_default())
}

pub struct LirFnBuilder {
    /// The function being built.
    product: lir::Function,
    /// The stack of instruction sequences that are currently being built. The
    /// innermost instruction sequence is at the top of the stack.
    insn_seqs: Vec<lir::InsnSeqId>,
    /// The InsnPc of the instruction for all function arguments.
    arguments_pc: lir::InsnPc,
    /// Maps the context ptr to a LIR value.
    context_ptr: Option<lir::ValRef>,
    /// Maps the environment ptr to a LIR value.
    env_ptr: Option<lir::ValRef>,
    /// Maps each local variable to a set of LIR values representing that
    /// local variable. How to interpret the values depends on the type of the
    /// local variable.
    local_values: HashMap<LocalId, SmallVec<[lir::ValRef; 1]>>,
}

impl LirFnBuilder {
    /// Creates a new LIR builder with an empty function that has no parameters,
    /// returns no values, and has a single body with a single FunctionArgs
    /// instruction.
    fn new() -> Self {
        let mut insn_seqs = TiVec::new();
        let mut main_insn_seq = TiVec::new();
        let args = main_insn_seq
            .push_and_get_key(lir::InsnKind::FunctionArgs { output_type: smallvec![] });
        let main_body = insn_seqs.push_and_get_key(main_insn_seq);
        Self {
            product: lir::Function {
                parameter_types: Vec::new(),
                body: lir::Block { output_type: smallvec![], body: main_body },
                insn_seqs,
                stack_space: 0,
                debug_fn_name: None,
                debug_val_names: HashMap::new(),
            },
            insn_seqs: vec![main_body],
            arguments_pc: lir::InsnPc(main_body, args),
            context_ptr: None,
            env_ptr: None,
            local_values: HashMap::new(),
        }
    }

    fn set_return_type(&mut self, tys: SmallVec<[lir::ValType; 1]>) {
        self.product.body.output_type = tys;
    }

    /// Adds a parameter to the function. Returns the index of the parameter.
    fn add_lir_parameter(&mut self, ty: lir::ValType) -> u8 {
        // add the parameter to the function's parameter list
        let param_idx = self.product.parameter_types.len() as u8;
        self.product.parameter_types.push(ty);

        // and to the FunctionArgs instruction in the main body
        let lir::InsnPc(seq_id, idx) = self.arguments_pc;
        let lir::InsnKind::FunctionArgs { output_type } = &mut self.product.insn_seqs[seq_id][idx]
        else {
            unreachable!(
                "the first instruction in the main body must be a FunctionArgs instruction"
            );
        };
        assert_eq!(output_type.len(), param_idx as usize);
        output_type.push(ty);

        param_idx
    }

    fn add_context_parameter(&mut self) {
        assert!(self.context_ptr.is_none());
        let param_idx = self.add_lir_parameter(lir::ValType::Ptr);
        self.context_ptr = Some(lir::ValRef(self.arguments_pc, param_idx));
    }

    fn add_env_parameter(&mut self) {
        assert!(self.env_ptr.is_none());
        let param_idx = self.add_lir_parameter(lir::ValType::Ptr);
        self.env_ptr = Some(lir::ValRef(self.arguments_pc, param_idx));
    }

    fn push_lir_insn(
        &mut self,
        insn: lir::InsnKind,
        num_output_vals: usize,
    ) -> SmallVec<[lir::ValRef; 1]> {
        // TODO this function should do deduplication if possible

        let insn_seq_id = *self.insn_seqs.last().unwrap();
        let insn_seq = &mut self.product.insn_seqs[insn_seq_id];

        let insn_idx = insn_seq.push_and_get_key(insn);

        (0..num_output_vals)
            .map(|i| lir::ValRef(lir::InsnPc(insn_seq_id, insn_idx), i as u8))
            .collect()
    }
}

pub fn translate_procedure(
    program_ctx: &LirProgramBuilder,
    mir: &Program,
    workspace: &Workspace,
    function_id: FunctionId,
) -> lir::Function {
    let Function {
        debug_name,
        takes_env,
        takes_context,
        parameters,
        return_value,
        locals,
        statements,
    } = &mir.functions[function_id];

    let mut fn_ctx = LirFnBuilder::new();

    fn_ctx.product.debug_fn_name = debug_name.clone();

    if let Some(return_value) = return_value {
        fn_ctx.set_return_type(locals[*return_value].ty.to_lir_type());
    }

    if *takes_env {
        fn_ctx.add_env_parameter();
    }
    if *takes_context {
        fn_ctx.add_context_parameter();
    }
    for parameter in parameters {
        let lir_params: SmallVec<[lir::ValType; 1]> = locals[*parameter].ty.to_lir_type();
        let mut lir_args = SmallVec::new();
        for param_ty in lir_params {
            let idx = fn_ctx.add_lir_parameter(param_ty);
            lir_args.push(lir::ValRef(fn_ctx.arguments_pc, idx));
        }
        fn_ctx.local_values.insert(*parameter, lir_args);
    }

    let StatementBlock { statements } = statements;

    for statement in statements {
        match statement {
            StatementKind::Op(op) => {
                translate_operator(mir, workspace, program_ctx, &mut fn_ctx, op);
            }
            StatementKind::IfElse { condition, then_block, else_block } => todo!(),
            StatementKind::Loop { block } => todo!(),
            StatementKind::Stop => todo!(),
        }
    }

    // add a break instruction to return the final value
    if let Some(return_value) = return_value {
        fn_ctx.push_lir_insn(
            lir::InsnKind::Break {
                target: lir::InsnSeqId(0),
                values: fn_ctx.local_values[return_value].iter().copied().collect(),
            },
            1,
        );
    }

    fn_ctx.product
}

fn translate_constant(constant: &Constant) -> SmallVec<[lir::Const; 1]> {
    let Constant { value } = constant;
    match value {
        UnpackedDynBox::Float(f) => {
            smallvec![lir::Const { r#type: lir::ValType::F64, value: f.to_bits() }]
        }
        UnpackedDynBox::Int(i) => {
            smallvec![lir::Const { r#type: lir::ValType::I64, value: *i as u64 }]
        }
        UnpackedDynBox::Bool(b) => {
            smallvec![lir::Const { r#type: lir::ValType::I8, value: *b as u64 }]
        }
        _ => todo!(),
    }
}

fn translate_operator(
    mir: &Program,
    workspace: &Workspace,
    program_ctx: &LirProgramBuilder,
    fn_ctx: &mut LirFnBuilder,
    op: &Operation,
) {
    let Operation { local_id, operator, args: mir_args } = op;

    let mut lir_args = mir_args_to_lir_args(program_ctx, fn_ctx, mir_args);

    let output_vals;
    match operator {
        Operator::UserFunctionCall { target } => {
            let lir_fn_id = program_ctx.available_user_functions[&target];

            // the env parameter cannot be present in a non-closure
            assert!(!mir.functions[*target].takes_env);
            // add the implicit context parameter
            if mir.functions[*target].takes_context {
                lir_args.insert(0, fn_ctx.context_ptr.expect("a function that calls another function with a context parameter must itself have a context parameter"));
            }

            let output_type = program_ctx.function_signatures[&lir_fn_id].1.clone();
            let num_output_vals = output_type.len();
            output_vals = fn_ctx.push_lir_insn(
                lir::InsnKind::CallUserFunction {
                    function: lir_fn_id,
                    output_type,
                    args: lir_args.into_boxed_slice(),
                },
                num_output_vals,
            );
        }
        Operator::CreateTurtles { breed } => {
            let lir_fn_id = program_ctx.available_host_functions["create_turtles"];

            // add the implicit context parameter
            lir_args.insert(
                0,
                fn_ctx.context_ptr.expect(
                    "the create_turtles host function must be called with a context parameter",
                ),
            );

            // add the breed argument
            let breed_id = fn_ctx.push_lir_insn(
                lir::InsnKind::Const(lir::Const {
                    r#type: lir::ValType::I64,
                    value: breed.data().as_ffi(),
                }),
                1,
            );
            lir_args.insert(1, breed_id[0]);

            output_vals = fn_ctx.push_lir_insn(
                lir::InsnKind::CallHostFunction {
                    function: lir_fn_id,
                    output_type: smallvec![],
                    args: lir_args.into_boxed_slice(),
                },
                0,
            );
        }
        Operator::HostFunctionCall { name: "clear_all" } => {
            let lir_fn_id = program_ctx.available_host_functions["clear_all"];

            // add the implicit context parameter
            lir_args.insert(
                0,
                fn_ctx
                    .context_ptr
                    .expect("the clear_all host function must be called with a context parameter"),
            );

            output_vals = fn_ctx.push_lir_insn(
                lir::InsnKind::CallHostFunction {
                    function: lir_fn_id,
                    output_type: smallvec![],
                    args: lir_args.into_boxed_slice(),
                },
                0,
            );
        }
        Operator::SetTurtleField { field } => {
            // TODO the value might take up more than one lir argument
            let &[agent_id, value] = &lir_args[..] else {
                unreachable!("the set field operator has two arguments");
            };

            // insert instruction to convert turtle id into an index
            let turtle_idx = fn_ctx.push_lir_insn(
                lir::InsnKind::UnaryOp { op: lir::UnaryOpcode::I64ToI32, operand: agent_id },
                1,
            );

            // insert instruction to convert context into workspace pointer
            let workspace_ptr = fn_ctx.push_lir_insn(
                lir::InsnKind::MemLoad {
                    r#type: lir::ValType::Ptr,
                    offset: std::mem::offset_of!(CanonExecutionContext, workspace),
                    ptr: fn_ctx.context_ptr.unwrap(),
                },
                1,
            );

            /// Returns the stride of the row and the offset of the field within
            /// the row, given an agent field descriptor applying to a set of
            /// row buffers.
            fn row_buffer_array_info(
                row_buffers: &[Option<RowBuffer>],
                field: AgentFieldDescriptor,
            ) -> (usize, usize) {
                let schema = row_buffers[usize::from(field.buffer_idx)]
                    .as_ref()
                    .expect("the row buffer containing the field must exist")
                    .schema();
                let field_offset = schema.field(usize::from(field.field_idx)).offset;
                (schema.stride(), field_offset)
            }
            let (row_buffer_idx, row_stride, field_offset) = match field {
                AgentField::Size => {
                    let base_data = workspace.world.turtles.turtle_schema().base_data();
                    let (stride, base_offset) =
                        row_buffer_array_info(workspace.world.turtles.row_buffers(), base_data);
                    (
                        usize::from(base_data.buffer_idx),
                        stride,
                        base_offset + offset_of!(TurtleBaseData, size),
                    )
                }
                AgentField::Color => {
                    let base_data = workspace.world.turtles.turtle_schema().base_data();
                    let (stride, base_offset) =
                        row_buffer_array_info(workspace.world.turtles.row_buffers(), base_data);
                    (
                        usize::from(base_data.buffer_idx),
                        stride,
                        base_offset + offset_of!(TurtleBaseData, color),
                    )
                }
                AgentField::Custom(field_id) => {
                    let (stride, field_offset) =
                        row_buffer_array_info(workspace.world.turtles.row_buffers(), *field_id);
                    (usize::from(field_id.buffer_idx), stride, field_offset)
                }
            };

            // insert instruction to convert workspace pointer into a pointer to
            // the correct rowbuffer
            let row_buffer_ptr = fn_ctx.push_lir_insn(
                lir::InsnKind::MemLoad {
                    r#type: lir::ValType::Ptr,
                    offset: offset_of!(Workspace, world)
                        + offset_of!(World, turtles)
                        + OFFSET_TURTLES_TO_DATA
                        + (row_buffer_idx * size_of::<Option<RowBuffer>>()),
                    ptr: workspace_ptr[0],
                },
                1,
            );

            // convert the rowbuffer pointer to a pointer to row
            let row_ptr = fn_ctx.push_lir_insn(
                lir::InsnKind::DeriveElement {
                    element_size: row_stride,
                    ptr: row_buffer_ptr[0],
                    index: turtle_idx[0],
                },
                1,
            );

            // insert an instruction to store the value into the row
            output_vals = fn_ctx.push_lir_insn(
                lir::InsnKind::MemStore { offset: field_offset, ptr: row_ptr[0], value },
                0,
            );
        }
        Operator::DirectOperator(DirectOperator::Identity) => {
            output_vals = lir_args.into_iter().collect();
        }
        _ => todo!(),
    }

    if let Some(local_id) = local_id {
        fn_ctx.local_values.insert(*local_id, output_vals);
    }
}

fn mir_args_to_lir_args(
    program_ctx: &LirProgramBuilder,
    fn_ctx: &mut LirFnBuilder,
    mir_args: &[Operand],
) -> Vec<lir::ValRef> {
    let mut lir_args = Vec::new();
    for mir_arg in mir_args {
        match mir_arg {
            Operand::Constant(c) => {
                let lir_consts = translate_constant(c);
                // push instructions for the constants onto the
                // stack
                for val in lir_consts {
                    let outputs = fn_ctx.push_lir_insn(lir::InsnKind::Const(val), 1);
                    lir_args.push(outputs[0]);
                }
            }
            Operand::LocalVar(local_id) => {
                for lir_value in &fn_ctx.local_values[local_id] {
                    lir_args.push(*lir_value);
                }
            }
            Operand::GlobalVar(_) => todo!(),
            Operand::ImmClosure(imm_closure) => {
                let ImmClosure { captures, body } = imm_closure;
                if captures.is_empty() {
                    // pass a dangling pointer as the environment
                    let outputs = fn_ctx.push_lir_insn(lir::InsnKind::Const(lir::Const::NULL), 1);
                    lir_args.push(outputs[0]);
                } else {
                    todo!(
                        "The captured variables need to be spilled to the stack and a reference to them created"
                    );
                }

                // pass the pointer of the function
                let outputs = fn_ctx.push_lir_insn(
                    lir::InsnKind::UserFunctionPtr {
                        function: program_ctx.available_user_functions[body],
                    },
                    1,
                );
                lir_args.push(outputs[0]);
            }
        }
    }

    lir_args
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::{
        mir::{DirectOperator, LocalDeclaration},
        sim::{
            agent_schema::{PatchSchema, TurtleSchema},
            patch::Patches,
            shapes::Shapes,
            tick::Tick,
            topology::{Topology, TopologySpec},
            turtle::{Breed, BreedId, Turtles},
        },
        util::{cell::RefCell, rng::CanonRng},
    };

    fn create_workspace() -> (Workspace, BreedId) {
        let topology_spec = TopologySpec {
            min_pxcor: -35,
            max_pycor: 35,
            patches_width: 71,
            patches_height: 71,
            wrap_x: false,
            wrap_y: false,
        };
        let patch_schema = PatchSchema::new(
            1,
            &[
                (NetlogoInternalType::FLOAT, 2),   // chemical
                (NetlogoInternalType::FLOAT, 0),   // food
                (NetlogoInternalType::BOOLEAN, 0), // nest?
                (NetlogoInternalType::FLOAT, 0),   // nest-scent
                (NetlogoInternalType::FLOAT, 0),   // food-source-number
            ],
            &[1, 2],
        );
        let turtle_schema = TurtleSchema::default();
        let patches = Patches::new(patch_schema, &topology_spec);
        let (turtle_breeds, default_turtle_breed) = {
            let mut breeds = SlotMap::with_key();
            let key = breeds.insert(Breed {
                name: Rc::from("turtles"),
                singular_name: Rc::from("turtle"),
                active_custom_fields: vec![],
            });
            (breeds, key)
        };
        let turtles = Turtles::new(turtle_schema, turtle_breeds);
        let topology = Topology::new(topology_spec);
        let tick_counter = Tick::default();
        let shapes = Shapes::default();
        let world = World { turtles, patches, topology, tick_counter, shapes };
        let rng = Rc::new(RefCell::new(CanonRng::new(0)));
        let workspace = Workspace { world, rng };

        // TODO declare the population widget variable

        (workspace, default_turtle_breed)
    }

    #[test]
    fn return_1() {
        let (workspace, default_turtle_breed) = create_workspace();
        let mut mir = Program::default();
        let mut locals = SlotMap::with_key();
        let return_value = locals.insert(LocalDeclaration {
            debug_name: Some("return_value".to_string()),
            mutable: true,
            ty: NetlogoInternalType::INTEGER,
        });
        let function = Function {
            debug_name: Some("return_1".to_string()),
            takes_env: false,
            takes_context: false,
            parameters: vec![],
            return_value: Some(return_value),
            locals,
            statements: StatementBlock {
                statements: vec![StatementKind::Op(Operation {
                    local_id: Some(return_value),
                    operator: Operator::DirectOperator(DirectOperator::Identity),
                    args: vec![Operand::Constant(Constant { value: UnpackedDynBox::Int(1) })],
                })],
            },
        };
        let function_id = mir.functions.insert(function);

        let mut program_ctx = LirProgramBuilder::new(&workspace, &mir);
        let lir_fn = translate_procedure(&mut program_ctx, &mir, &workspace, function_id);
        println!("{:#?}", lir_fn);
    }

    #[test]
    fn call_host_function_with_imm_closure() {
        let (workspace, default_turtle_breed) = create_workspace();

        let mut program = Program::default();

        let setup_body_0 = {
            let mut locals = SlotMap::with_key();
            // let local_context = locals.insert(mir::LocalDeclaration {
            //     debug_name: Some("context".to_string()),
            //     mutable: false,
            // })
            let local_self = locals.insert(LocalDeclaration {
                debug_name: Some("self".to_string()),
                mutable: false,
                ty: NetlogoInternalType::TURTLE_ID,
            });
            Function {
                debug_name: Some("setup/body_0".to_string()),
                takes_env: true,
                takes_context: true,
                parameters: vec![local_self],
                return_value: None,
                locals,
                statements: StatementBlock {
                    statements: vec![
                        StatementKind::Op(Operation {
                            local_id: None,
                            operator: Operator::SetTurtleField { field: AgentField::Size },
                            args: vec![
                                Operand::LocalVar(local_self),
                                Operand::Constant(Constant { value: UnpackedDynBox::Float(2.0) }),
                            ],
                        }),
                        StatementKind::Op(Operation {
                            local_id: None,
                            operator: Operator::SetTurtleField { field: AgentField::Color },
                            args: vec![Operand::Constant(Constant {
                                value: UnpackedDynBox::Float(15.0),
                            })],
                        }),
                    ],
                },
            }
        };
        let setup_body_0 = program.functions.insert(setup_body_0);

        let setup = Function {
            debug_name: Some("setup".to_string()),
            takes_env: false,
            takes_context: true,
            parameters: vec![],
            return_value: None,
            locals: SlotMap::with_key(),
            statements: StatementBlock {
                statements: vec![
                    StatementKind::Op(Operation {
                        local_id: None,
                        operator: Operator::HostFunctionCall { name: "clear_all" },
                        args: vec![],
                    }),
                    StatementKind::Op(Operation {
                        local_id: None,
                        operator: Operator::CreateTurtles { breed: default_turtle_breed },
                        args: vec![
                            // TODO make this use a global variable
                            Operand::Constant(Constant { value: UnpackedDynBox::Int(125) }),
                            Operand::ImmClosure(ImmClosure {
                                captures: vec![],
                                body: setup_body_0,
                            }),
                        ],
                    }),
                ],
            },
        };
        let setup = program.functions.insert(setup);

        let mut program_ctx = LirProgramBuilder::new(&workspace, &program);
        let lir_fn = translate_procedure(&mut program_ctx, &program, &workspace, setup);
        println!("{:#?}", lir_fn);
    }
}
