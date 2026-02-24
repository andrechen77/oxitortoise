use std::collections::HashMap;

use crate::{
    hir::{self, Node},
    mir::{
        self, Block, CtrlFlowConstruct, ElementaryStatement, IfElse, LocalId, Operation, Place,
        PlaceOperand, Statement,
    },
};

#[derive(Default)]
struct MirProgramBuilder {
    functions: HashMap<mir::FunctionId, mir::Function>,
    next_function_id: u32,
    next_local_id: u32,
    next_ctrl_flow_construct_id: u32,
}

impl MirProgramBuilder {
    fn new() -> Self {
        Default::default()
    }

    fn create_function(&mut self) -> MirFunctionBuilder<'_> {
        MirFunctionBuilder::new(self)
    }

    fn next_local_id(&mut self) -> mir::LocalId {
        let id = mir::LocalId(self.next_local_id);
        self.next_local_id += 1;
        id
    }

    fn next_function_id(&mut self) -> mir::FunctionId {
        let id = mir::FunctionId(self.next_function_id);
        self.next_function_id += 1;
        id
    }

    fn next_ctrl_flow_construct_id(&mut self) -> mir::ControlFlowConstructId {
        let id = mir::ControlFlowConstructId(self.next_ctrl_flow_construct_id);
        self.next_ctrl_flow_construct_id += 1;
        id
    }
}

pub struct MirFunctionBuilder<'a> {
    program_builder: &'a mut MirProgramBuilder,
    fn_id: mir::FunctionId,
    locals: HashMap<mir::LocalId, mir::LocalDecl>,
    return_local: Option<mir::LocalId>,
    ctrl_flow_construct_ids: HashMap<hir::NodeId, (mir::ControlFlowConstructId, mir::LocalId)>,
    hir_local_to_mir_local: HashMap<hir::LocalId, mir::LocalId>,
    statements_out: Vec<Statement>,
}

impl<'a> MirFunctionBuilder<'a> {
    fn new(program_builder: &'a mut MirProgramBuilder) -> Self {
        let fn_id = program_builder.next_function_id();
        Self {
            program_builder,
            fn_id,
            locals: HashMap::new(),
            return_local: None,
            ctrl_flow_construct_ids: HashMap::new(),
            hir_local_to_mir_local: HashMap::new(),
            statements_out: Vec::new(),
        }
    }

    /// Finishes the function builder and adds the function to the program builder.
    fn finish(mut self) {
        let body = if self.statements_out.len() == 1 {
            self.statements_out.pop().expect("we checked that the length is 1")
        } else {
            todo!()
        };
        let function = mir::Function {
            local_decls: self.locals,
            return_local: self.return_local.expect("return local must be set"),
            body,
        };
        self.program_builder.functions.insert(self.fn_id, function);
    }

    fn set_return(&mut self, local: mir::LocalId) {
        let old = self.return_local.replace(local);
        assert!(old.is_none(), "return local cannot be set twice");
    }

    fn create_local(&mut self, decl: mir::LocalDecl) -> mir::LocalId {
        let id = self.program_builder.next_local_id();
        self.locals.insert(id, decl);
        id
    }

    fn create_ctrl_flow_construct_id(
        &mut self,
        hir_node_id: hir::NodeId,
        dst_local: mir::LocalId,
    ) -> mir::ControlFlowConstructId {
        let id = self.program_builder.next_ctrl_flow_construct_id();
        self.ctrl_flow_construct_ids.insert(hir_node_id, (id, dst_local.into()));
        id
    }

    pub fn add_operation(&mut self, dst: mir::Place, op: Operation) {
        self.statements_out.push(Statement::Elementary(ElementaryStatement::Assign { dst, op }));
    }

    fn with_inner_statement_seq(&mut self, f: impl FnOnce(&mut Self)) -> Vec<Statement> {
        let old = std::mem::take(&mut self.statements_out);
        f(self);
        std::mem::replace(&mut self.statements_out, old)
    }

    pub fn add_block(
        &mut self,
        node_id: hir::NodeId,
        output_local: mir::LocalId,
        f: impl FnOnce(&mut Self),
    ) {
        let statements = self.with_inner_statement_seq(f);
        let block = Statement::CtrlFlow(CtrlFlowConstruct::Block(Block {
            id: self.create_ctrl_flow_construct_id(node_id, output_local),
            statements,
        }));
        self.statements_out.push(block);
    }

    pub fn add_if_else(
        &mut self,
        node_id: hir::NodeId,
        output_local: mir::LocalId,
        condition: mir::Place,
        f_then: impl FnOnce(&mut Self),
        f_else: impl FnOnce(&mut Self),
    ) {
        let then_block = self.with_inner_statement_seq(f_then);
        let else_block = self.with_inner_statement_seq(f_else);
        let stmt = Statement::CtrlFlow(CtrlFlowConstruct::IfElse(IfElse {
            id: self.create_ctrl_flow_construct_id(node_id, output_local),
            condition,
            then_block,
            else_block,
        }));
        self.statements_out.push(stmt);
    }

    pub fn add_break(&mut self, target: hir::NodeId, value: Option<mir::Place>) {
        let (target, dst) = self.ctrl_flow_construct_ids[&target];

        // if the break evaluates to a value, we need to assign it to the target block's output
        if let Some(value) = value {
            self.statements_out.push(Statement::Elementary(ElementaryStatement::Assign {
                dst: dst.into(),
                op: Operation::Operand(PlaceOperand::Move(value)),
            }))
        }

        self.statements_out.push(Statement::Elementary(ElementaryStatement::Break { target }));
    }

    /// Generates a MIR statements to evaluate the given HIR node with all its
    /// dependencies. The return value as well as any temporary values created
    /// during evaluation are stored in new local variables; this function does not
    /// reuse existing locals.
    ///
    /// There is currently no checking if dependencies have already been evaluated.
    pub fn translate_hir_node(&mut self, hir: &hir::Program, node_id: hir::NodeId) -> LocalId {
        let node = &hir.nodes[node_id];
        let output_ty = node.output_type(hir).repr();

        // the node's output will be stored in this local variable
        let output_local = self.create_local(mir::LocalDecl { debug_name: None, ty: output_ty });

        hir.nodes[node_id].write_mir_execution(hir, node_id, self, output_local);

        output_local
    }
}

pub fn hir_to_mir(hir: &hir::Program) -> mir::Program {
    let mut builder = MirProgramBuilder::new();

    // iterate through each function and convert it to an MIR function
    for (hir_fn_id, hir_fn) in &hir.functions {
        let mut fn_builder = builder.create_function();

        // create a MIR local for each HIR local variable
        for hir_local in &hir_fn.locals {
            let hir_local_decl = &hir.locals[*hir_local];
            let local_id = fn_builder.create_local(mir::LocalDecl {
                debug_name: hir_local_decl.debug_name.clone(),
                ty: hir_local_decl.ty.repr(),
            });
            fn_builder.hir_local_to_mir_local.insert(*hir_local, local_id);
        }

        // add all the nodes to the function body
        let node_id = hir_fn.root_node;
        let return_place = fn_builder.translate_hir_node(hir, node_id);
        fn_builder.set_return(return_place);
        fn_builder.finish();
    }

    todo!()
}
