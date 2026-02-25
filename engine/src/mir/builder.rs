use std::collections::HashMap;

use crate::mir::{
    self, ElementaryStatement, Function, FunctionId, Label, LocalId, Operation, Statement,
};

#[derive(Default)]
pub struct ProgramBuilder {
    functions: HashMap<FunctionId, Function>,
    next_function_id: u32,
    next_local_id: u32,
    next_label: u32,
}

impl ProgramBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn create_function(&mut self) -> FunctionBuilder<'_> {
        FunctionBuilder::new(self)
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

    fn next_label(&mut self) -> mir::Label {
        let id = mir::Label(self.next_label);
        self.next_label += 1;
        id
    }
}

pub struct FunctionBuilder<'a> {
    program_builder: &'a mut ProgramBuilder,
    fn_id: FunctionId,
    locals: HashMap<LocalId, mir::LocalDecl>,
    return_local: Option<LocalId>,
    statements_out: Vec<Statement>,
}

impl<'a> FunctionBuilder<'a> {
    fn new(program_builder: &'a mut ProgramBuilder) -> Self {
        let fn_id = program_builder.next_function_id();
        Self {
            program_builder,
            fn_id,
            locals: HashMap::new(),
            return_local: None,
            statements_out: Vec::new(),
        }
    }

    /// Finishes the function builder and adds the function to the program builder.
    pub fn finish(mut self) {
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

    pub fn set_return(&mut self, local: LocalId) {
        let old = self.return_local.replace(local);
        assert!(old.is_none(), "return local cannot be set twice");
    }

    pub fn create_local(&mut self, decl: mir::LocalDecl) -> LocalId {
        let id = self.program_builder.next_local_id();
        self.locals.insert(id, decl);
        id
    }

    pub fn create_label(&mut self) -> Label {
        self.program_builder.next_label()
    }

    pub fn add_operation_with_dst(&mut self, dst: mir::Place, op: Operation) {
        self.statements_out.push(Statement::Elementary(ElementaryStatement::Assign { dst, op }));
    }

    pub fn add_operation(&mut self, local_decl: mir::LocalDecl, op: Operation) -> LocalId {
        let dst = self.create_local(local_decl);
        self.add_operation_with_dst(dst.into(), op);
        dst
    }

    /// If the statement is an assignment from an operation, consider using
    /// [`FunctionBuilder::add_operation`] instead.
    pub fn add_statement(&mut self, stmt: Statement) {
        self.statements_out.push(stmt);
    }

    pub fn with_inner_statement_seq<T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> T,
    ) -> (Vec<Statement>, T) {
        let old = std::mem::take(&mut self.statements_out);
        let result = f(self);
        let stmts = std::mem::replace(&mut self.statements_out, old);
        (stmts, result)
    }
}
