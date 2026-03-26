use pretty_print::PrettyPrinter;

use crate::hir::{Expr as _, Function, NameContext, Program};
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
            function_bodies,
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
                        write!(p, "{:?}#{:?}", fn_id, functions[fn_id].debug_name)?;
                        Ok(())
                    },
                    |p, (fn_id, function)| {
                        let Function { debug_name: _, parameters, return_ty, is_entrypoint } =
                            function;
                        p.add_struct("Function", |p| {
                            p.add_field("is_entrypoint", is_entrypoint)?;
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
                            p.add_field_with("return_ty", |p| write!(p, "{}", return_ty))?;
                            p.add_field_with("body", |p| {
                                function_bodies[fn_id].pretty_print(
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
