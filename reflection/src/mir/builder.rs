use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use tracing::{trace, warn};

use crate::{
    DynType, Reflect,
    dyn_type::ProjectionError,
    mir::{
        ElementaryStatement, Function, FunctionId, Label, LocalDecl, LocalId, Operation, Place,
        PlaceOperand, Program, Statement, consolidate_statements,
    },
    static_type::CloneKind,
};

#[derive(Default)]
pub struct ProgramBuilder {
    functions: BTreeMap<FunctionId, Function>,
    function_stubs: BTreeMap<FunctionId, FunctionStub>,
    next_function_id: u32,
    next_local_id: u32,
    next_label: u32,
}

pub struct FunctionStub {
    pub return_ty: DynType,
}

impl ProgramBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn finish(self) -> Program {
        let Self {
            functions,
            function_stubs,
            next_function_id: _,
            next_local_id: _,
            next_label: _,
        } = self;
        assert!(function_stubs.is_empty(), "function stubs must be finished");
        Program { functions }
    }

    pub fn completed_function(&self, id: FunctionId) -> Option<&Function> {
        self.functions.get(&id)
    }

    pub fn function_stub(&self, fn_id: FunctionId) -> Option<&FunctionStub> {
        self.function_stubs.get(&fn_id)
    }

    pub fn create_function(
        &mut self,
        id: FunctionId,
        debug_name: Option<Arc<str>>,
    ) -> FunctionBuilder<'_> {
        FunctionBuilder::new(self, id, debug_name)
    }

    pub fn next_function_id(&mut self) -> FunctionId {
        let id = FunctionId(self.next_function_id);
        self.next_function_id += 1;
        id
    }

    pub fn insert_function_stub(&mut self, fn_id: FunctionId, stub: FunctionStub) {
        self.function_stubs.insert(fn_id, stub);
    }

    fn next_local_id(&mut self) -> LocalId {
        let id = LocalId(self.next_local_id);
        self.next_local_id += 1;
        id
    }

    fn next_label(&mut self) -> Label {
        let id = Label(self.next_label);
        self.next_label += 1;
        id
    }
}

pub struct FunctionBuilder<'a> {
    program_builder: &'a mut ProgramBuilder,
    debug_name: Option<Arc<str>>,
    fn_id: FunctionId,
    parameters: Vec<LocalId>,
    locals: BTreeMap<LocalId, LocalDecl>,
    // TODO(wishlist) consider replacing this with a more granular data structure
    // that can track moves in and out of iniividual fields. then we would also
    // not need to manually assert when a local is fully initialized.
    currently_init_locals: BTreeSet<LocalId>,
    return_local: Option<LocalId>,
    statements_out: Vec<Statement>,
    unit_local: Option<LocalId>,
}

impl<'a> FunctionBuilder<'a> {
    fn new(
        program_builder: &'a mut ProgramBuilder,
        fn_id: FunctionId,
        debug_name: Option<Arc<str>>,
    ) -> Self {
        trace!("new function builder for function {:?}", fn_id);
        let mut new = Self {
            debug_name,
            program_builder,
            fn_id,
            locals: BTreeMap::new(),
            currently_init_locals: BTreeSet::new(),
            parameters: Vec::new(),
            return_local: None,
            statements_out: Vec::new(),
            unit_local: None,
        };
        new.unit_local =
            Some(new.create_local(LocalDecl { debug_name: None, ty: <()>::dyn_type() }));
        new
    }

    pub fn create_another_function(&mut self) -> FunctionBuilder<'_> {
        let fn_id = self.program_builder.next_function_id();
        FunctionBuilder::new(self.program_builder, fn_id, None)
    }

    /// Finishes the function builder and adds the function to the program builder.
    pub fn finish(self) -> FunctionId {
        let body =
            consolidate_statements(self.statements_out, || self.program_builder.next_label());
        let function = Function {
            debug_name: self.debug_name,
            parameters: self.parameters,
            local_decls: self.locals,
            return_local: self.return_local.expect("return local must be set"),
            body,
        };
        self.program_builder.functions.insert(self.fn_id, function);
        self.program_builder.function_stubs.remove(&self.fn_id);
        self.fn_id
    }

    pub fn set_return(&mut self, local: LocalId) {
        let old = self.return_local.replace(local);
        assert!(old.is_none(), "return local cannot be set twice");
    }

    pub fn unit_local(&self) -> LocalId {
        self.unit_local.expect("unit local must be set").into()
    }

    pub fn create_parameter(&mut self, decl: LocalDecl) -> LocalId {
        let id = self.create_local(decl);
        self.parameters.push(id);
        self.set_as_init(id);
        trace!("created parameter: {:?} {:?}", id, self.locals[&id]);
        id
    }

    pub fn create_local(&mut self, decl: LocalDecl) -> LocalId {
        let id = self.program_builder.next_local_id();
        self.locals.insert(id, decl);
        id
    }

    pub fn get_local_mut(&mut self, id: LocalId) -> &mut LocalDecl {
        self.locals.get_mut(&id).expect("local must be declared")
    }

    pub fn is_init(&self, local: LocalId) -> bool {
        self.currently_init_locals.contains(&local)
    }

    pub fn set_as_init(&mut self, local: LocalId) {
        self.currently_init_locals.insert(local);
    }

    pub fn type_of_place(&self, place: &Place) -> &DynType {
        let local_ty = &self.locals[&place.local].ty;
        let result = place
            .projections
            .iter()
            .fold(Ok(local_ty), |ty, &projection| ty.and_then(|ty| ty.project(projection)));
        match result {
            Ok(ty) => ty,
            Err(ProjectionError) => {
                panic!("projection error: place {:?} does not exist in type {:?}", place, local_ty);
            }
        }
    }

    pub fn type_of_op(&self, op: &Operation) -> DynType {
        match op {
            Operation::Operand(operand) => match operand {
                PlaceOperand::Copy(place) => self.type_of_place(place).clone(),
                PlaceOperand::Move(local) => self.type_of_place(&local.place()).clone(),
                PlaceOperand::Borrow(place) => DynType::ref_to(self.type_of_place(place).clone()),
            },
            Operation::Const(pod_value) => pod_value.ty().clone(),
            Operation::FunctionPtr { function: _ } => {
                // we can probably get away with making no assertions about the type
                DynType::default()
            }
            Operation::CallHostFunction { function, .. } => {
                (*function.return_type.dyn_type).clone()
            }
            Operation::CallUserFunction { function, .. } => self
                .program_builder
                .completed_function(*function)
                .map(|f| f.return_ty().clone())
                .unwrap_or_else(|| {
                    self.program_builder.function_stub(*function).unwrap().return_ty.clone()
                }),
            // Operation::BinaryOp { opcode, lhs: _, rhs: _ } => match opcode {
            //     lir::BinaryOpcode::FAdd
            //     | lir::BinaryOpcode::FSub
            //     | lir::BinaryOpcode::FMul
            //     | lir::BinaryOpcode::FDiv => NlFloat::mir_type(),
            //     lir::BinaryOpcode::FLt
            //     | lir::BinaryOpcode::FGt
            //     | lir::BinaryOpcode::FLte
            //     | lir::BinaryOpcode::FGte
            //     | lir::BinaryOpcode::FEq
            //     | lir::BinaryOpcode::IEq
            //     | lir::BinaryOpcode::INeq => bool::mir_type(),
            //     lir::BinaryOpcode::And | lir::BinaryOpcode::Or => bool::mir_type(),
            //     _ => panic!("unsupported binary opcode: {:?}", opcode),
            // },
            // Operation::UnaryOp { opcode, operand: _ } => match opcode {
            //     lir::UnaryOpcode::Not => bool::mir_type(),
            //     lir::UnaryOpcode::FNeg => NlFloat::mir_type(),
            //     lir::UnaryOpcode::I64ToI32 => u32::mir_type(),
            // },
            _ => todo!(),
        }
    }

    pub fn create_label(&mut self) -> Label {
        self.program_builder.next_label()
    }

    pub fn add_operation_with_dst(&mut self, dst: Place, op: Operation) {
        // make sure that the types match
        let dst_ty = self.type_of_place(&dst);
        let val_ty = self.type_of_op(&op);
        if !dst_ty.is_supertype_of(&val_ty) {
            warn!(
                "type of operation does not match type of destination place: {:?} := {:?}",
                dst_ty, val_ty
            );
        }

        self.add_statement(Statement::Elementary(ElementaryStatement::Assign { dst, op }));
    }

    pub fn add_operation_with_decl(&mut self, local_decl: LocalDecl, op: Operation) -> LocalId {
        // make sure that the types match
        let val_ty = self.type_of_op(&op);
        if !local_decl.ty.is_supertype_of(&val_ty) {
            warn!(
                "type of operation does not match type of local declaration: {:?} := {:?}",
                local_decl.ty, local_decl.ty
            );
        }

        let dst = self.create_local(local_decl);
        self.add_statement(Statement::Elementary(ElementaryStatement::Assign {
            dst: dst.place(),
            op,
        }));
        dst
    }

    pub fn add_operation(&mut self, name: Option<Arc<str>>, op: Operation) -> LocalId {
        let local_decl = LocalDecl { debug_name: name, ty: self.type_of_op(&op) };
        self.add_operation_with_decl(local_decl, op)
    }

    /// If the statement is an assignment from an operation, consider using
    /// [`FunctionBuilder::add_operation`] instead.
    pub fn add_statement(&mut self, stmt: Statement) {
        // process updates to the currently initialized locals
        match &stmt {
            Statement::Elementary(ElementaryStatement::Assign { dst, op }) => {
                if dst.projections.is_empty() {
                    self.currently_init_locals.insert(dst.unwrap_local());
                }

                for operand in op.operands() {
                    if let PlaceOperand::Move(local) = operand {
                        // move out of the local
                        let old_exists = self.currently_init_locals.remove(&local);
                        if !old_exists && !self.type_of_place(&local.place()).is::<()>() {
                            warn!("moving out of uninitialized local: {:?}", local);
                        }
                    }
                }
            }
            Statement::Elementary(ElementaryStatement::Drop { src }) => {
                if src.projections.is_empty() {
                    let old_exists = self.currently_init_locals.remove(&src.unwrap_local());

                    if !old_exists {
                        warn!("dropping uninitialized local: {:?}", src);
                    }
                }
            }
            _ => {}
        }

        self.statements_out.push(stmt);
    }

    /// Moves a value from one place to another. This may potentially destroy the
    /// source place if it is not Copy. The destination place is considered
    /// initialized and will be deinitialized before the value is moved in.
    pub fn move_to_init(&mut self, dst_init: Place, src: LocalId) {
        // deinitialize the destination place
        self.add_statement(Statement::Elementary(ElementaryStatement::Drop {
            src: dst_init.clone(),
        }));

        // move the value into the place
        self.add_operation_with_dst(dst_init, Operation::Operand(PlaceOperand::Move(src)));
    }

    pub fn clone_to_uninit(&mut self, src: Place, dst: Place) {
        match self.type_of_place(&src).clone_kind() {
            CloneKind::Copy => {
                self.add_operation_with_dst(dst, Operation::Operand(PlaceOperand::Copy(src)))
            }
            CloneKind::Dynamic { clone_fn_info, .. } => self.add_operation_with_dst(
                dst,
                Operation::CallHostFunction {
                    function: clone_fn_info,
                    args: vec![PlaceOperand::Borrow(src)],
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
            self.create_local(LocalDecl { debug_name: None, ty: self.type_of_place(&src).clone() });
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
