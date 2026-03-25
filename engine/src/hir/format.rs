use pretty_print::PrettyPrinter;

use crate::hir::{Expr as _, Function, LocalDecl, LocalId, Program};
use std::{collections::BTreeMap, fmt::Write};

#[derive(Debug, Clone, Copy)]
pub enum NameContext<'a> {
    Global(&'a Program),
    Local { local_vars: &'a BTreeMap<LocalId, LocalDecl>, parent: &'a NameContext<'a> },
}

impl<'a> NameContext<'a> {
    pub fn from_program(program: &'a Program) -> Self {
        NameContext::Global(program)
    }

    pub fn with_locals(&'a self, local_vars: &'a BTreeMap<LocalId, LocalDecl>) -> Self {
        NameContext::Local { local_vars, parent: self }
    }

    pub fn program(&self) -> &'a Program {
        match self {
            NameContext::Global(program) => program,
            NameContext::Local { parent, .. } => parent.program(),
        }
    }

    pub fn lookup_local_var(&self, local_id: LocalId) -> Option<&'a LocalDecl> {
        match self {
            NameContext::Global(_program) => None,
            NameContext::Local { local_vars, parent } => {
                local_vars.get(&local_id).or_else(|| parent.lookup_local_var(local_id))
            }
        }
    }
}

impl Program {
    pub fn pretty_print(&self) -> String {
        let mut out = String::new();
        let mut printer = PrettyPrinter::new(&mut out);

        let Program {
            global_vars,
            turtle_breeds,
            custom_turtle_vars,
            custom_patch_vars,
            functions,
        } = self;

        let _ = printer.add_struct("Program", |p| {
            p.add_field_with("global_vars", |p| {
                p.add_map(
                    global_vars.iter().enumerate(),
                    |p, index| write!(p, "{}", index),
                    |p, (_, global)| write!(p, "{:?}", global),
                )
            })?;
            p.add_field_with("turtle_breeds", |p| {
                p.add_map(
                    turtle_breeds.iter(),
                    |p, breed_id| write!(p, "{:?}", breed_id),
                    |p, (_, breed)| write!(p, "{:?}", breed),
                )
            })?;
            p.add_field_with("custom_turtle_vars", |p| {
                p.add_map(
                    custom_turtle_vars.iter().enumerate(),
                    |p, index| write!(p, "{}", index),
                    |p, (_, var)| write!(p, "{:?}", var),
                )
            })?;
            p.add_field_with("custom_patch_vars", |p| {
                p.add_map(
                    custom_patch_vars.iter().enumerate(),
                    |p, index| write!(p, "{}", index),
                    |p, (_, var)| write!(p, "{:?}", var),
                )
            })?;
            p.add_field_with("functions", |p| {
                p.add_map(
                    functions.iter(),
                    |p, fn_id| {
                        write!(p, "{:?}", fn_id)?;
                        if let Some(debug_name) = &functions[fn_id].debug_name {
                            write!(p, " ({:?})", debug_name)?;
                        }
                        Ok(())
                    },
                    |p, (_, function)| {
                        let Function { debug_name: _, parameters, body } = function;
                        p.add_struct("Function", |p| {
                            p.add_field_with("parameters", |p| {
                                p.indented(|p| {
                                    // add local declarations
                                    for (local_id, decl) in parameters {
                                        p.line()?;
                                        write!(
                                            p,
                                            "{:?} {}: {},",
                                            local_id,
                                            decl.debug_name.as_ref().map_or("", |n| n.as_ref()),
                                            decl.ty
                                        )?;
                                    }
                                    Ok(())
                                })
                            })?;
                            p.add_field_with("body", |p| {
                                body.pretty_print(
                                    p,
                                    NameContext::from_program(self).with_locals(parameters),
                                )
                            })?;
                            Ok(())
                        })
                    },
                )
            })
        });

        out
    }
}
