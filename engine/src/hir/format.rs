use pretty_print::PrettyPrinter;

use crate::hir::{Expr as _, Function, Program};
use std::fmt::Write;

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
                            p.add_field_with("body", |p| body.pretty_print(p, self))?;
                            Ok(())
                        })
                    },
                )
            })
        });

        out
    }
}
