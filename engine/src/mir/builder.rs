use std::{collections::BTreeMap, sync::Arc};

use crate::{
    mir::{
        self, ElementaryStatement, Function, FunctionId, Label, LocalId, MirType, MirTypeContents,
        MirTypeInfo, Operation, Place, PlaceOperand, Statement,
    },
    util::reflection::{CloneKind, ReflectComponents as _},
};

#[derive(Default)]
pub struct ProgramBuilder {
    functions: BTreeMap<FunctionId, Function>,
    next_function_id: u32,
    next_local_id: u32,
    next_label: u32,
}

impl ProgramBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn function(&self, id: FunctionId) -> &Function {
        &self.functions[&id]
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
    parameters: Vec<LocalId>,
    locals: BTreeMap<LocalId, mir::LocalDecl>,
    return_local: Option<LocalId>,
    statements_out: Vec<Statement>,
    unit_local: Option<LocalId>,
}

impl<'a> FunctionBuilder<'a> {
    fn new(program_builder: &'a mut ProgramBuilder) -> Self {
        let fn_id = program_builder.next_function_id();
        let mut new = Self {
            program_builder,
            fn_id,
            locals: BTreeMap::new(),
            parameters: Vec::new(),
            return_local: None,
            statements_out: Vec::new(),
            unit_local: None,
        };
        new.unit_local =
            Some(new.create_local(mir::LocalDecl { debug_name: None, ty: <()>::mir_type() }));
        new
    }

    pub fn create_another_function(&mut self) -> FunctionBuilder<'_> {
        FunctionBuilder::new(self.program_builder)
    }

    /// Finishes the function builder and adds the function to the program builder.
    pub fn finish(mut self) -> FunctionId {
        let body = if self.statements_out.len() == 1 {
            self.statements_out.pop().expect("we checked that the length is 1")
        } else {
            todo!()
        };
        let function = mir::Function {
            parameters: self.parameters,
            local_decls: self.locals,
            return_local: self.return_local.expect("return local must be set"),
            body,
        };
        self.program_builder.functions.insert(self.fn_id, function);
        self.fn_id
    }

    pub fn set_return(&mut self, local: LocalId) {
        let old = self.return_local.replace(local);
        assert!(old.is_none(), "return local cannot be set twice");
    }

    pub fn unit_local(&self) -> LocalId {
        self.unit_local.expect("unit local must be set").into()
    }

    pub fn create_parameter(&mut self, decl: mir::LocalDecl) -> LocalId {
        let id = self.create_local(decl);
        self.parameters.push(id);
        id
    }

    pub fn create_local(&mut self, decl: mir::LocalDecl) -> LocalId {
        let id = self.program_builder.next_local_id();
        self.locals.insert(id, decl);
        id
    }

    pub fn get_local_mut(&mut self, id: LocalId) -> &mut mir::LocalDecl {
        self.locals.get_mut(&id).expect("local must be declared")
    }

    pub fn is_init(&self, local: LocalId) -> bool {
        let _ = local;
        todo!("TODO implement is_init")
    }

    pub fn type_of_place(&self, place: &Place) -> mir::MirType {
        let local_ty = &self.locals[&place.local].ty;
        place
            .projections
            .iter()
            .fold(local_ty, |ty, &projection| ty.contents.project(projection))
            .clone()
    }

    pub fn type_of_op(&self, op: &Operation) -> MirType {
        match op {
            Operation::Operand(operand) => match operand {
                PlaceOperand::Copy(place) => self.type_of_place(place),
                PlaceOperand::Move(local) => self.type_of_place(&local.place()),
                PlaceOperand::Borrow(place) => {
                    let place_ty = self.type_of_place(place);
                    Arc::new(MirTypeInfo {
                        static_ty: None,
                        contents: MirTypeContents::IsPointerTo(place_ty),
                    })
                }
            },
            Operation::Const { value } => (value.ty().make_mir_type)(),
            Operation::FunctionPtr { function: _ } => {
                // we can probably get away with making no assertions about the type
                MirType::default()
            }
            Operation::CallHostFunction { function, .. } => (function.return_type.make_mir_type)(),
            Operation::CallUserFunction { function, .. } => {
                self.program_builder.function(*function).return_ty().clone()
            }
            Operation::BinaryOp { .. } => {
                unimplemented!("hardcoded binary ops will be sunsetted in favor of host fn calls")
            }
            Operation::UnaryOp { .. } => {
                unimplemented!("hardcoded unary ops will be sunsetted in favor of host fn calls")
            }
        }
    }

    pub fn create_label(&mut self) -> Label {
        self.program_builder.next_label()
    }

    pub fn add_operation_with_dst(&mut self, dst: mir::Place, op: Operation) {
        // make sure that the types match
        let ty = self.type_of_op(&op);
        assert_eq!(
            ty,
            self.type_of_place(&dst),
            "type of operation must match type of destination place"
        );

        self.statements_out.push(Statement::Elementary(ElementaryStatement::Assign { dst, op }));
    }

    pub fn add_operation_with_decl(
        &mut self,
        local_decl: mir::LocalDecl,
        op: Operation,
    ) -> LocalId {
        // make sure that the types match
        let ty = self.type_of_op(&op);
        assert_eq!(ty, local_decl.ty, "type of operation must match type of local declaration");

        let dst = self.create_local(local_decl);
        self.add_operation_with_dst(dst.into(), op);
        // TODO could do something with the type assertion here
        dst
    }

    pub fn add_operation(&mut self, name: Option<Arc<str>>, op: Operation) -> LocalId {
        let local_decl = mir::LocalDecl { debug_name: name, ty: self.type_of_op(&op) };
        self.add_operation_with_decl(local_decl, op)
    }

    /// If the statement is an assignment from an operation, consider using
    /// [`FunctionBuilder::add_operation`] instead.
    pub fn add_statement(&mut self, stmt: Statement) {
        self.statements_out.push(stmt);
    }

    /// Moves a value from one place to another. This may potentially destroy the
    /// source place if it is not Copy. The destination place is considered
    /// initialized and will be deinitialized before the value is moved in.
    pub fn move_to_init(&mut self, dst_init: Place, src: LocalId) {
        // deinitialize the destination place
        self.add_statement(mir::Statement::Elementary(mir::ElementaryStatement::Drop {
            src: dst_init.clone(),
        }));

        // move the value into the place
        self.add_operation_with_dst(
            dst_init,
            mir::Operation::Operand(mir::PlaceOperand::Move(src)),
        );
    }

    pub fn clone_to_uninit(&mut self, src: Place, dst: Place) {
        let clone_kind = &self.type_of_place(&src).static_ty.unwrap().clone;
        match clone_kind {
            CloneKind::Copy => self
                .add_operation_with_dst(dst, mir::Operation::Operand(mir::PlaceOperand::Copy(src))),
            CloneKind::Dynamic { clone_fn_info, .. } => self.add_operation_with_dst(
                dst,
                mir::Operation::CallHostFunction {
                    function: clone_fn_info,
                    args: vec![mir::PlaceOperand::Borrow(src)],
                },
            ),
            CloneKind::None => {
                panic!("Cannot clone a value that is neither Copy nor Clone");
            }
        }
    }

    /// Moves a value from one place to another. The source place is not
    /// deinitialized. The destination place will not be deinitialized (i.e. it is
    /// assumed to be uninitialized). Useful for loading variables from memory.
    pub fn clone_to_new(&mut self, src: Place) -> LocalId {
        let dst =
            self.create_local(mir::LocalDecl { debug_name: None, ty: self.type_of_place(&src) });
        self.clone_to_uninit(src, dst.place());
        dst
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
