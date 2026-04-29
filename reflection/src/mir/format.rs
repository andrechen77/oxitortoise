use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Write},
};

use pretty_print::PrettyPrinter;

use crate::mir::{
    Block, CtrlFlowConstruct, ElementaryStatement, Function, FunctionId, IfElse, LocalDecl,
    LocalId, Operation, Place, PlaceOperand, Program, Projection, Statement,
};

impl Program {
    pub fn pretty_print(&self) -> String {
        let mut out = String::new();
        let mut p = PrettyPrinter::new(&mut out);

        let Program { functions } = self;

        let _ = p.add_struct("Program", |p| {
            p.add_field_with("functions", |p| {
                p.add_map(
                    functions.iter(),
                    |p, fn_id| {
                        if let Some(debug_name) = &functions[fn_id].debug_name {
                            write!(p, "{:?}#{}", fn_id, debug_name)
                        } else {
                            write!(p, "{:?}", fn_id)
                        }
                    },
                    |p, (_, function)| function.pretty_print(p, functions),
                )
            })
        });

        out
    }
}

impl Function {
    pub fn pretty_print(
        &self,
        p: &mut PrettyPrinter<impl Write>,
        functions: &BTreeMap<FunctionId, Function>,
    ) -> fmt::Result {
        let Function { debug_name: _, parameters, local_decls, return_local, body } = self;

        p.add_struct("Function", |p| {
            let mut declared_locals = BTreeSet::new();

            // return value
            p.line()?;
            write_local_decl(p, "return", *return_local, &local_decls[return_local])?;
            declared_locals.insert(*return_local);

            // parameters
            for param in parameters {
                p.line()?;
                write_local_decl(p, "param", *param, &local_decls[param])?;
                declared_locals.insert(*param);
            }

            // body
            body.pretty_print(p, &local_decls, &mut declared_locals, functions)
        })
    }
}

impl Statement {
    pub fn pretty_print(
        &self,
        p: &mut PrettyPrinter<impl Write>,
        local_decls: &BTreeMap<LocalId, LocalDecl>,
        declared_locals: &mut BTreeSet<LocalId>,
        functions: &BTreeMap<FunctionId, Function>,
    ) -> fmt::Result {
        p.line()?;
        match self {
            Statement::CtrlFlow(CtrlFlowConstruct::Block(Block { label, statements })) => {
                write!(p, "{:?}: ", label)?;
                p.add_struct("", |p| {
                    for statement in statements {
                        statement.pretty_print(p, local_decls, declared_locals, functions)?;
                    }
                    Ok(())
                })
            }
            Statement::CtrlFlow(CtrlFlowConstruct::IfElse(IfElse { condition, then, r#else })) => {
                write!(p, "if ")?;
                condition.pretty_print(p, local_decls)?;
                write!(p, " ")?;
                p.add_struct("", |p| {
                    then.pretty_print(p, local_decls, declared_locals, functions)?;
                    Ok(())
                })?;
                write!(p, " else ")?;
                p.add_struct("", |p| {
                    r#else.pretty_print(p, local_decls, declared_locals, functions)?;
                    Ok(())
                })
            }
            Statement::Elementary(ElementaryStatement::Drop { src }) => {
                write!(p, "drop ")?;
                src.pretty_print(p, local_decls)?;
                write!(p, ";")
            }
            Statement::Elementary(ElementaryStatement::Assign { dst, op }) => {
                let needs_declare = declared_locals.insert(dst.local);
                if needs_declare {
                    let local_ty = &local_decls[&dst.local].ty;
                    if dst.projections.is_empty() {
                        // use the "let local: ty = initializer;" form.
                        write!(p, "let ")?;
                        dst.pretty_print(p, local_decls)?;
                        write!(p, ": {:?} = ", local_ty)?;
                        op.pretty_print(p, local_decls, functions)?;
                        write!(p, ";")
                    } else {
                        // use the "let local: ty; place = initializer" form
                        write!(p, "let ")?;
                        dst.local.pretty_print(p, local_decls)?;
                        write!(p, ": {:?};", local_ty)?;
                        p.line()?;
                        dst.pretty_print(p, local_decls)?;
                        write!(p, " = ")?;
                        op.pretty_print(p, local_decls, functions)?;
                        write!(p, ";")
                    }
                } else {
                    // just do "place = initializer"
                    dst.pretty_print(p, local_decls)?;
                    write!(p, " = ")?;
                    op.pretty_print(p, local_decls, functions)?;
                    write!(p, ";")
                }
            }
            Statement::Elementary(ElementaryStatement::Break { target }) => {
                write!(p, "break {:?};", target)
            }
            _ => todo!(),
        }
    }
}

fn write_local_decl(
    p: &mut PrettyPrinter<impl Write>,
    prefix: &str,
    local_id: LocalId,
    decl: &LocalDecl,
) -> fmt::Result {
    if let Some(debug_name) = &decl.debug_name {
        write!(p, "{} {:?}#{}: {:?};", prefix, local_id, debug_name, decl.ty)
    } else {
        write!(p, "{} {:?}: {:?};", prefix, local_id, decl.ty)
    }
}

impl LocalId {
    pub fn pretty_print(
        &self,
        p: &mut PrettyPrinter<impl Write>,
        local_decls: &BTreeMap<LocalId, LocalDecl>,
    ) -> fmt::Result {
        if let Some(debug_name) = &local_decls[self].debug_name {
            write!(p, "{:?}#{}", self, debug_name)
        } else {
            write!(p, "{:?}", self)
        }
    }
}

impl Place {
    pub fn pretty_print(
        &self,
        p: &mut PrettyPrinter<impl Write>,
        local_decls: &BTreeMap<LocalId, LocalDecl>,
    ) -> fmt::Result {
        let Place { local, projections } = self;
        local.pretty_print(p, local_decls)?;

        for projection in projections {
            match projection {
                Projection::Deref => write!(p, ".deref")?,
                Projection::Field { byte_offset } => write!(p, ".({})", byte_offset)?,
                Projection::DynamicIndex(index) => write!(p, ".[{index:?}]")?,
                Projection::StaticIndex(index) => write!(p, ".[{index}]")?,
            }
        }
        Ok(())
    }
}

impl PlaceOperand {
    pub fn pretty_print(
        &self,
        p: &mut PrettyPrinter<impl Write>,
        local_decls: &BTreeMap<LocalId, LocalDecl>,
    ) -> fmt::Result {
        match self {
            PlaceOperand::Move(local) => {
                write!(p, "move ")?;
                local.pretty_print(p, local_decls)
            }
            PlaceOperand::Copy(place) => {
                write!(p, "copy ")?;
                place.pretty_print(p, local_decls)
            }
            PlaceOperand::Borrow(place) => {
                write!(p, "borrow ")?;
                place.pretty_print(p, local_decls)
            }
        }
    }
}

impl Operation {
    pub fn pretty_print(
        &self,
        p: &mut PrettyPrinter<impl Write>,
        local_decls: &BTreeMap<LocalId, LocalDecl>,
        functions: &BTreeMap<FunctionId, Function>,
    ) -> fmt::Result {
        match self {
            Operation::Operand(operand) => operand.pretty_print(p, local_decls),
            Operation::Const(pod_value) => {
                write!(p, "{:?} {:?}", pod_value.ty(), pod_value.bytes())
            }
            Operation::FunctionPtr { function } => {
                if let Some(debug_name) = &functions[function].debug_name {
                    write!(p, "{:?}#{}", function, debug_name)
                } else {
                    write!(p, "{:?}", function)
                }
            }
            Operation::BinaryOp { opcode, lhs, rhs } => {
                lhs.pretty_print(p, local_decls)?;
                write!(p, " {opcode:?} ")?;
                rhs.pretty_print(p, local_decls)
            }
            Operation::UnaryOp { opcode, operand } => {
                write!(p, "{opcode:?} ")?;
                operand.pretty_print(p, local_decls)
            }
            Operation::CallUserFunction { function, args } => {
                if let Some(debug_name) = &functions[function].debug_name {
                    write!(p, "{:?}#{}", function, debug_name)?;
                } else {
                    write!(p, "{:?}", function)?;
                }
                p.add_fn_call("", |p| {
                    for arg in args {
                        p.add_fn_arg_with(|p| arg.pretty_print(p, local_decls))?;
                    }
                    Ok(())
                })
            }
            Operation::CallHostFunction { function, args } => {
                write!(p, "{:?}", function)?;
                p.add_fn_call("", |p| {
                    for arg in args {
                        p.add_fn_arg_with(|p| arg.pretty_print(p, local_decls))?;
                    }
                    Ok(())
                })
            }
        }
    }
}

impl fmt::Debug for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operation::Operand(operand) => write!(f, "{:?}", operand),
            Operation::Const(pod_value) => {
                write!(f, "{:?} {:?}", pod_value.ty(), pod_value.bytes())
            }
            Operation::FunctionPtr { function } => write!(f, "{:?}", function),
            Operation::BinaryOp { opcode, lhs, rhs } => write!(f, "{lhs:?} {opcode:?} {rhs:?}"),
            Operation::UnaryOp { opcode, operand } => write!(f, "{opcode:?} {operand:?}"),
            Operation::CallUserFunction { function, args } => {
                write!(f, "{:?}(", function)?;
                for arg in args {
                    write!(f, "{:?}, ", arg)?;
                }
                write!(f, ")")
            }
            Operation::CallHostFunction { function, args } => {
                write!(f, "{:?}(", function)?;
                for arg in args {
                    write!(f, "{:?}, ", arg)?;
                }
                write!(f, ")")
            }
        }
    }
}
