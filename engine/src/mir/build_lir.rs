//! These functions are used on a fully optimized MIR program to translate it
//! into LIR. No optimization is performed here. Each MIR function will
//! translate to a single LIR function.

use std::collections::HashMap;

use lir::{smallvec::SmallVec, typed_index_collections::TiVec};
use slotmap::{SecondaryMap, SlotMap};

use crate::{
    exec::jit::HOST_FUNCTIONS,
    mir::{self, EffectfulNode},
};

#[derive(Debug)]
pub struct LirProgramBuilder {
    pub available_user_functions: HashMap<mir::FunctionId, lir::FunctionId>,
    pub host_function_ids: HostFunctionIds,
    pub function_signatures:
        HashMap<lir::FunctionId, (Vec<lir::ValType>, SmallVec<[lir::ValType; 1]>)>,
}

pub fn mir_to_lir(mir: &mir::Program) -> lir::Program {
    // generate host function signatures (always the same)
    let (host_functions, host_function_ids) = HOST_FUNCTIONS.clone();

    let mut builder = LirProgramBuilder {
        available_user_functions: HashMap::new(),
        host_function_ids,
        function_signatures: HashMap::new(),
    };

    // translate all user function signatures. we collect these first because
    // functions might reference each other, and their signatures are required
    let mut user_function_tracker = SlotMap::with_key(); // used only to allocate lir::FunctionId
    for (mir_fn_id, mir_fn) in mir.functions.iter() {
        let signature = translate_function_signature(&*mir_fn.borrow());
        // allocate a new function id for the LIR function
        let lir_fn_id = user_function_tracker.insert(());
        builder.available_user_functions.insert(mir_fn_id, lir_fn_id);
        builder.function_signatures.insert(lir_fn_id, signature);
    }

    // translate all user function bodies
    let mut lir_fn_bodies = SecondaryMap::new();
    for (mir_fn_id, mir_fn) in mir.functions.iter() {
        let lir_fn = translate_function_body(&*mir_fn.borrow(), &mut builder);
        let lir_fn_id = builder.available_user_functions[&mir_fn_id];
        lir_fn_bodies.insert(lir_fn_id, lir_fn);
    }

    lir::Program {
        entrypoints: vec![], // TODO choose entrypoints, probably add a field to MIR program
        user_functions: lir_fn_bodies,
        host_functions,
    }
}

fn translate_function_signature(
    function: &mir::Function,
) -> (Vec<lir::ValType>, SmallVec<[lir::ValType; 1]>) {
    let mut params = Vec::new();
    for parameter in &function.parameters {
        params.extend(function.locals[*parameter].ty.repr().to_lir_type());
    }
    let return_value = function.return_ty.repr().to_lir_type();
    (params, return_value)
}

#[derive(Debug)]
pub struct LirInsnBuilder<'a> {
    pub program_builder: &'a LirProgramBuilder,
    /// Maps MIR local variables to their LIR representations.
    pub local_to_lir: HashMap<mir::LocalId, LocalLocation>,
    /// Maps node ids to LIR values. This also doubles as a record of which
    /// nodes have been executed; it should map to an empty vector for nodes
    /// that don't have any outputs.
    pub node_to_lir: HashMap<mir::NodeId, SmallVec<[lir::ValRef; 1]>>,
    /// The LIR function being built.
    pub product: lir::Function,
    /// The stack of current instruction sequence being built.
    pub insn_seqs: Vec<lir::InsnSeqId>,
}

impl<'a> LirInsnBuilder<'a> {
    pub fn get_node_results(
        &mut self,
        nodes: &SlotMap<mir::NodeId, Box<dyn EffectfulNode>>,
        node_id: mir::NodeId,
    ) -> &[lir::ValRef] {
        if !self.node_to_lir.contains_key(&node_id) {
            nodes[node_id].write_lir_execution(node_id, nodes, self).unwrap();
        }
        self.node_to_lir.get(&node_id).unwrap().as_slice()
    }
}

/// Describes how an MIR local variable is stored in LIR.
#[derive(Debug)]
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

#[derive(Debug, Clone, Copy)]
pub struct HostFunctionIds {
    pub clear_all: lir::HostFunctionId,
    pub reset_ticks: lir::HostFunctionId,
    pub create_turtles: lir::HostFunctionId,
    pub ask_all_turtles: lir::HostFunctionId,
}

fn translate_function_body(
    function: &mir::Function,
    program_builder: &mut LirProgramBuilder,
) -> lir::Function {
    // turn the MIR local variables into LIR local variables
    let mut lir_debug_var_names = HashMap::new();
    let mut local_to_lir = HashMap::new();
    let mut lir_local_var_types = TiVec::new();
    for &local_id in &function.parameters {
        let local_decl = &function.locals[local_id];
        assert_eq!(local_decl.storage, mir::LocalStorage::Register);

        let &[lir_type] = local_decl.ty.repr().to_lir_type().as_slice() else {
            todo!("handle local variables that take up multiple LIR values")
        };
        let lir_var_id = lir_local_var_types.push_and_get_key(lir_type);
        local_to_lir.insert(local_id, LocalLocation::Var { var_id: lir_var_id });
        if let Some(debug_name) = local_decl.debug_name.clone() {
            lir_debug_var_names.insert(lir_var_id, debug_name);
        }
    }
    let num_lir_parameters = lir_local_var_types.len();
    for (local_id, local_decl) in &function.locals {
        if local_to_lir.contains_key(&local_id) {
            continue;
        }
        match local_decl.storage {
            mir::LocalStorage::Register => {
                let &[lir_type] = local_decl.ty.repr().to_lir_type().as_slice() else {
                    todo!("handle local variables that take up multiple LIR values")
                };
                let lir_var_id = lir_local_var_types.push_and_get_key(lir_type);
                local_to_lir.insert(local_id, LocalLocation::Var { var_id: lir_var_id });
                if let Some(debug_name) = local_decl.debug_name.clone() {
                    lir_debug_var_names.insert(lir_var_id, debug_name);
                }
            }
            mir::LocalStorage::Stack => todo!(),
        }
    }

    // initialize the LIR function and its associated metadata
    let mut insn_seqs = TiVec::new();
    let main_body = insn_seqs.push_and_get_key(TiVec::new());
    let body_block =
        lir::Block { output_type: function.return_ty.repr().to_lir_type(), body: main_body };
    let lir_function = lir::Function {
        local_vars: lir_local_var_types,
        num_parameters: num_lir_parameters,
        stack_space: 0, // TODO add stack variables
        body: body_block,
        insn_seqs,
        debug_fn_name: function.debug_name.clone(),
        debug_val_names: HashMap::new(), // TODO add debug val names
        debug_var_names: lir_debug_var_names,
    };
    let mut fn_builder = LirInsnBuilder {
        program_builder,
        node_to_lir: HashMap::new(),
        local_to_lir,
        product: lir_function,
        insn_seqs: vec![main_body],
    };

    // algorithm idea: iterate over all statements. if a node depends on another
    // node which is not rooted in its own statement, then those dependencies
    // are translated first
    translate_stmt_block(&*function.nodes.borrow(), &mut fn_builder, &function.cfg);

    fn translate_stmt_block(
        nodes: &SlotMap<mir::NodeId, Box<dyn EffectfulNode>>,
        fn_builder: &mut LirInsnBuilder,
        stmt_block: &mir::StatementBlock,
    ) {
        for stmt in &stmt_block.statements {
            match stmt {
                &mir::StatementKind::Node(node_id) => {
                    nodes[node_id].write_lir_execution(node_id, nodes, fn_builder).inspect_err(|_| {
                        eprintln!("failed to translate node {:?} to LIR", nodes[node_id]);
                    }).expect(
                        "by the time we get to translating to LIR, all nodes should be able to convert to LIR",
                    );
                }
                mir::StatementKind::IfElse { condition, then_block, else_block } => {
                    let _ = condition;
                    let _ = then_block;
                    let _ = else_block;
                    todo!()
                }
                mir::StatementKind::Stop => todo!(),
                _ => todo!(),
            }
        }
    }

    fn_builder.product
}
