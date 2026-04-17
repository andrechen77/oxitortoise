use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Write},
};

use pretty_print::PrettyPrinter;

use crate::mir::{
    Block, CtrlFlowConstruct, ElementaryStatement, Function, IfElse, LocalDecl, LocalId, Operation,
    Program, Statement,
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
                    |p, fn_id| write!(p, "{:?}", fn_id),
                    |p, (_, function)| function.pretty_print(p),
                )
            })
        });

        out
    }
}

impl Function {
    pub fn pretty_print(&self, p: &mut PrettyPrinter<impl Write>) -> fmt::Result {
        let Function { parameters, local_decls, return_local, body } = self;

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
            body.pretty_print(p, &local_decls, &mut declared_locals)
        })
    }
}

impl Statement {
    pub fn pretty_print(
        &self,
        p: &mut PrettyPrinter<impl Write>,
        locals_decls: &BTreeMap<LocalId, LocalDecl>,
        declared_locals: &mut BTreeSet<LocalId>,
    ) -> fmt::Result {
        p.line()?;
        match self {
            Statement::CtrlFlow(CtrlFlowConstruct::Block(Block { label, statements })) => {
                write!(p, "{:?}: ", label)?;
                p.add_struct("", |p| {
                    for statement in statements {
                        statement.pretty_print(p, locals_decls, declared_locals)?;
                    }
                    Ok(())
                })
            }
            Statement::CtrlFlow(CtrlFlowConstruct::IfElse(IfElse { condition, then, r#else })) => {
                write!(p, "if {:?} ", condition)?;
                p.add_struct("", |p| {
                    then.pretty_print(p, locals_decls, declared_locals)?;
                    Ok(())
                })?;
                write!(p, " else ")?;
                p.add_struct("", |p| {
                    r#else.pretty_print(p, locals_decls, declared_locals)?;
                    Ok(())
                })
            }
            Statement::Elementary(ElementaryStatement::Drop { src }) => {
                write!(p, "drop {:?};", src)
            }
            Statement::Elementary(ElementaryStatement::Assign { dst, op }) => {
                let needs_declare = declared_locals.insert(dst.local);
                if needs_declare {
                    let local_ty = &locals_decls[&dst.local].ty;
                    if dst.projections.is_empty() {
                        // use the "let local: ty = initializer;" form.
                        write!(p, "let {:?}: {:?} = {:?},", dst, local_ty, op)
                    } else {
                        // use the "let local: ty; place = initializer" form
                        write!(p, "let {:?} {:?};", dst.local, local_ty)?;
                        p.line()?;
                        write!(p, "{:?} = {:?};", dst, op)
                    }
                } else {
                    // just do "place = initializer"
                    write!(p, "{:?} = {:?};", dst, op)
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
    write!(p, "{} {:?}#{:?}: {:?};", prefix, local_id, decl.debug_name, decl.ty)
}

impl fmt::Debug for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operation::Operand(operand) => write!(f, "{:?}", operand),
            Operation::Const { value } => write!(f, "{:?}", value),
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
