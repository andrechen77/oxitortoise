//! These functions are used on a fully optimized MIR program to translate it
//! into LIR. No optimization is performed here. Each MIR function will
//! translate to a single LIR function.

use std::{collections::HashMap, sync::Arc};

use lir::{
    smallvec::{SmallVec, smallvec},
    typed_index_collections::{TiVec, ti_vec},
};
use slotmap::{SecondaryMap, SlotMap};
use tracing::{error, instrument, trace};

use crate::{
    exec::jit::InstallLir,
    mir::{self, Node},
};

#[derive(Debug)]
pub struct LirProgramBuilder {
    pub available_user_functions: HashMap<mir::FunctionId, lir::FunctionId>,
    pub function_signatures:
        HashMap<lir::FunctionId, (Vec<lir::ValType>, SmallVec<[lir::ValType; 1]>)>,
}

pub fn mir_to_lir<I: InstallLir>(
    mir: &mir::Program,
) -> (lir::Program, HashMap<mir::FunctionId, lir::FunctionId>) {
    let mut builder = LirProgramBuilder {
        available_user_functions: HashMap::new(),
        function_signatures: HashMap::new(),
    };

    // translate all user function signatures. we collect these first because
    // functions might reference each other, and their signatures are required
    let mut user_function_tracker = SlotMap::with_key(); // used only to allocate lir::FunctionId
    for mir_fn_id in mir.functions.keys() {
        trace!("translating function signature for {}", mir_fn_id);
        let signature = translate_function_signature(mir, mir_fn_id);
        // allocate a new function id for the LIR function
        let lir_fn_id = user_function_tracker.insert(());
        builder.available_user_functions.insert(mir_fn_id, lir_fn_id);
        builder.function_signatures.insert(lir_fn_id, signature);
    }

    // translate all user function bodies
    let mut lir_fn_bodies = SecondaryMap::new();
    for mir_fn_id in mir.functions.keys() {
        trace!("translating function body for {}", mir_fn_id);
        let lir_fn = translate_function_body::<I>(mir, mir_fn_id, &mut builder);
        let lir_fn_id = builder.available_user_functions[&mir_fn_id];
        lir_fn_bodies.insert(lir_fn_id, lir_fn);
    }

    // add entrypoint shim functions
    let mut entrypoints = HashMap::new();
    for mir_fn_id in mir.functions.keys().filter(|id| mir.functions[*id].is_entrypoint) {
        let shim = create_entrypoint_shim(mir, mir_fn_id, &mut builder);
        if let Some(shim) = shim {
            // allocate a new function id for the shim function
            let lir_fn_id = user_function_tracker.insert(());
            lir_fn_bodies.insert(lir_fn_id, shim);
            entrypoints.insert(mir_fn_id, lir_fn_id);
        }
    }

    (lir::Program { user_functions: lir_fn_bodies }, entrypoints)
}

#[instrument(skip(program))]
fn translate_function_signature(
    program: &mir::Program,
    fn_id: mir::FunctionId,
) -> (Vec<lir::ValType>, SmallVec<[lir::ValType; 1]>) {
    let function = &program.functions[fn_id];

    let mut params = Vec::new();
    for parameter in &function.parameters {
        trace!("adding parameter {:?} with type {:?}", parameter, program.locals[*parameter].ty);
        params.extend(
            program.locals[*parameter]
                .ty
                .repr()
                .info()
                .mem_repr
                .expect("function parameter must have known ABI")
                .iter()
                .map(|&(_, r#type)| r#type.loaded_type()),
        );
    }
    trace!("adding return value with type {:?}", function.return_ty);
    let return_value = function
        .return_ty
        .repr()
        .info()
        .mem_repr
        .expect("function return type must have known ABI")
        .iter()
        .map(|&(_, r#type)| r#type.loaded_type())
        .collect();
    (params, return_value)
}

#[derive(Debug)]
pub struct LirInsnBuilder<'a> {
    pub program_builder: &'a LirProgramBuilder,
    /// The MIR function ID being translated.
    pub fn_id: mir::FunctionId,
    /// Maps MIR local variables to their LIR representations.
    pub local_to_lir: HashMap<mir::LocalId, LocalLocation>,
    /// Maps node ids to LIR values. This also doubles as a record of which
    /// nodes have been executed; it should map to an empty vector for nodes
    /// that were executed but don't have any outputs.
    pub node_to_lir: HashMap<mir::NodeId, SmallVec<[lir::ValRef; 1]>>,
    /// The LIR function being built.
    pub product: lir::Function,
    /// The stack of current instruction sequences being built.
    pub insn_seqs: Vec<lir::InsnSeqId>,
    /// Maps each block node to the LIR instruction sequence ID that should be
    /// broken from when the block is exited.
    pub block_to_insn_seq: HashMap<mir::NodeId, lir::InsnSeqId>,
}

impl<'a> LirInsnBuilder<'a> {
    pub fn get_node_results<I: InstallLir>(
        &mut self,
        program: &mir::Program,
        node_id: mir::NodeId,
    ) -> &[lir::ValRef] {
        if !self.node_to_lir.contains_key(&node_id) {
            let node = &program.nodes[node_id];
            trace!("writing LIR execution for node {:?} {:?}", node_id, node);
            node.write_lir_execution::<I>(program, node_id, self).unwrap_or_else(|e| {
                error!("failed to translate node {:?} to LIR: {:?}", node_id, e);
            });
        }
        self.node_to_lir
            .get(&node_id)
            .unwrap_or_else(|| {
                panic!("node {:?} should have made its LIR results available but did not", node_id)
            })
            .as_slice()
    }

    pub fn with_insn_seq<R>(
        &mut self,
        insn_seq: lir::InsnSeqId,
        op: impl FnOnce(&mut LirInsnBuilder) -> R,
    ) -> R {
        self.insn_seqs.push(insn_seq);
        let result = op(self);
        self.insn_seqs.pop();
        result
    }
}

/// Describes how an MIR local variable is stored in LIR.
#[derive(Debug, Copy, Clone)]
pub enum LocalLocation {
    /// The MIR local variable is stored on the stack at the specified offset.
    Stack { offset: usize },
    /// The MIR local variable is stored as an LIR local variable, starting at
    /// the specified index.
    Var { var_id: lir::VarId },
}

impl<'a> LirInsnBuilder<'a> {
    /// Helper function to push a LIR instruction. Returns a list of LIR values
    /// created by the instruction.
    pub fn push_lir_insn(&mut self, insn: lir::InsnKind) -> lir::InsnPc {
        let insn_seq_id = *self.insn_seqs.last().unwrap();
        let insn_seq = &mut self.product.insn_seqs[insn_seq_id];

        let insn_idx = insn_seq.push_and_get_key(insn);
        lir::InsnPc(insn_seq_id, insn_idx)
    }
}

#[instrument(skip(program, program_builder))]
fn translate_function_body<I: InstallLir>(
    program: &mir::Program,
    fn_id: mir::FunctionId,
    program_builder: &mut LirProgramBuilder,
) -> lir::Function {
    let function = &program.functions[fn_id];

    // turn the MIR local variables into LIR local variables
    let mut lir_debug_var_names = HashMap::new();
    let mut local_to_lir = HashMap::new();
    let mut lir_local_var_types = TiVec::new();
    fn add_local(
        program: &mir::Program,
        local_to_lir: &mut HashMap<mir::LocalId, LocalLocation>,
        lir_local_var_types: &mut TiVec<lir::VarId, lir::ValType>,
        lir_debug_var_names: &mut HashMap<lir::VarId, Arc<str>>,
        local_id: mir::LocalId,
    ) {
        let local_decl = &program.locals[local_id];
        match local_decl.storage {
            mir::LocalStorage::Register => {
                let lir_types: SmallVec<[lir::ValType; 1]> = local_decl
                    .ty
                    .repr()
                    .info()
                    .mem_repr
                    .expect("local variable must have known ABI")
                    .iter()
                    .map(|&(_, r#type)| r#type.loaded_type())
                    .collect();
                let &[lir_type] = lir_types.as_slice() else {
                    unimplemented!("handle local variables that take up multiple LIR values")
                };
                let lir_var_id = lir_local_var_types.push_and_get_key(lir_type);
                local_to_lir.insert(local_id, LocalLocation::Var { var_id: lir_var_id });
                if let Some(debug_name) = local_decl.debug_name.clone() {
                    lir_debug_var_names.insert(lir_var_id, debug_name);
                }
            }
            mir::LocalStorage::Stack => {
                todo!("TODO(mvp) allocate space on the stack and map the local variable to it")
            }
        }
    }
    for &param_id in &function.parameters {
        add_local(
            program,
            &mut local_to_lir,
            &mut lir_local_var_types,
            &mut lir_debug_var_names,
            param_id,
        );
    }
    let num_lir_parameters = lir_local_var_types.len();
    for &local_id in &function.locals {
        if !function.parameters.contains(&local_id) {
            add_local(
                program,
                &mut local_to_lir,
                &mut lir_local_var_types,
                &mut lir_debug_var_names,
                local_id,
            );
        }
    }

    // initialize the LIR function and its associated metadata
    let mut insn_seqs = TiVec::new();
    let main_body = insn_seqs.push_and_get_key(TiVec::new());
    let body_block = lir::Block {
        output_type: function
            .return_ty
            .repr()
            .info()
            .mem_repr
            .expect("function return type must have known ABI")
            .iter()
            .map(|&(_, r#type)| r#type.loaded_type())
            .collect(),
        body: main_body,
    };
    let lir_function = lir::Function {
        local_vars: lir_local_var_types,
        num_parameters: num_lir_parameters,
        stack_space: 0, // TODO(mvp) add stack variables
        body: body_block,
        insn_seqs,
        debug_fn_name: function.debug_name.clone(),
        debug_val_names: HashMap::new(), // TODO(nice_to_have) add debug val names
        debug_var_names: lir_debug_var_names,
        is_entrypoint: false, // even if the MIR function was an entrypoint, the real entrypoint is a shim function created later
    };
    let mut fn_builder = LirInsnBuilder {
        program_builder,
        fn_id,
        node_to_lir: HashMap::new(),
        local_to_lir,
        product: lir_function,
        insn_seqs: vec![main_body],
        block_to_insn_seq: HashMap::new(),
    };

    // translate the function body
    program.nodes[function.root_node]
        .write_lir_execution::<I>(program, function.root_node, &mut fn_builder)
        .unwrap();

    fn_builder.product
}

// returns None if we don't know how to create the shim.
// TODO complete this function so that it never returns None.
fn create_entrypoint_shim(
    program: &mir::Program,
    target_fn_id: mir::FunctionId,
    program_builder: &mut LirProgramBuilder,
) -> Option<lir::Function> {
    // TODO document the assumptions that are made here about parameter types,
    // order, etc, and ideally couple this function to the code that relies on
    // those assumptions (e.g. the definition of JitEntrypoint).

    // context pointer, pointer to variadic args, and number of args
    let params: TiVec<lir::VarId, lir::ValType> =
        ti_vec![lir::ValType::Ptr, lir::ValType::Ptr, lir::ValType::I32];

    let (target_arg_types, target_returns) = translate_function_signature(program, target_fn_id);

    let mut target_args = Vec::new();

    let mut insn_seqs: TiVec<lir::InsnSeqId, TiVec<lir::InsnIdx, lir::InsnKind>> = TiVec::new();
    let main_seq_id = insn_seqs.push_and_get_key(TiVec::new());
    let main_seq = &mut insn_seqs[main_seq_id];

    // check the number of non-context arguments
    let num_args = main_seq.push_and_get_key(lir::InsnKind::VarLoad { var_id: 2.into() });
    let num_args = lir::ValRef(lir::InsnPc(main_seq_id, num_args), 0);
    let expected_num_args = main_seq.push_and_get_key(lir::InsnKind::Const(lir::Value::I32(
        u32::try_from(target_arg_types.len()).unwrap() - 1,
    )));
    let expected_num_args = lir::ValRef(lir::InsnPc(main_seq_id, expected_num_args), 0);
    let is_num_args_correct = main_seq.push_and_get_key(lir::InsnKind::BinaryOp {
        op: lir::BinaryOpcode::INeq,
        lhs: num_args,
        rhs: expected_num_args,
    });
    let is_num_args_correct = lir::ValRef(lir::InsnPc(main_seq_id, is_num_args_correct), 0);
    // this could probably be a panic or unreachable but this works for now
    main_seq.push_and_get_key(lir::InsnKind::ConditionalBreak {
        target: main_seq_id,
        condition: is_num_args_correct,
        values: Box::new([]),
    });

    // load context argument
    let context = main_seq.push_and_get_key(lir::InsnKind::VarLoad { var_id: 0.into() });
    let context = lir::ValRef(lir::InsnPc(main_seq_id, context), 0);
    target_args.push(context);

    // we would load variadic arguments here. however, this would require
    // converting the array of PackedAny to a sequence of values which possibly
    // might be some other types, which is not yet implemented. there used to be
    // code here that just loads the values at the types declared by the target
    // function, when in fact they would be passed as PackedAny
    if target_arg_types.len() != 1 {
        return None;
    }

    // push the call to the target function
    main_seq.push_and_get_key(lir::InsnKind::CallUserFunction {
        function: program_builder.available_user_functions[&target_fn_id],
        output_type: target_returns,
        args: target_args.into_boxed_slice(),
    });

    // push a break instruction that returns unit
    main_seq
        .push_and_get_key(lir::InsnKind::Break { target: lir::InsnSeqId(0), values: Box::new([]) });

    let num_parameters = params.len();
    Some(lir::Function {
        local_vars: params,
        num_parameters,
        stack_space: 0,
        body: lir::Block { output_type: smallvec![], body: lir::InsnSeqId(0) },
        insn_seqs,
        debug_val_names: HashMap::new(),
        debug_var_names: HashMap::new(), // TODO
        is_entrypoint: true,
        debug_fn_name: Some(format!("entrypoint_shim {}", target_fn_id).into()),
    })
}
