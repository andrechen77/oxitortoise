use std::{
    fmt::{self, Write},
    ops::Add,
};

use super::*;

use pretty_print::PrettyPrinter;

impl Program {
    pub fn pretty_print(&self) -> String {
        let mut out = String::new();
        let mut p = PrettyPrinter::new(&mut out);
        let Program { user_functions } = self;
        let _ = p.add_struct("Lir", |p| {
            p.add_field("user_functions", |p| {
                p.add_map(
                    user_functions.iter(),
                    |p, fn_id| {
                        write!(p, "{:?}", fn_id)?;
                        if let Some(debug_fn_name) = &user_functions[fn_id].debug_fn_name {
                            write!(p, " ({:?})", debug_fn_name)?;
                        }
                        Ok(())
                    },
                    |p, (_, function)| {
                        let Function {
                            local_vars,
                            num_parameters,
                            stack_space,
                            body,
                            debug_fn_name,
                            ..
                        } = function;
                        p.add_struct("Function", |p| {
                            p.add_field("debug_name", |p| write!(p, "{:?}", debug_fn_name))?;
                            p.add_field("num_parameters", |p| write!(p, "{}", num_parameters))?;
                            p.add_field("local_vars", |p| {
                                p.add_list(
                                    local_vars.iter_enumerated(),
                                    |p, (local_var_id, local_var_ty)| {
                                        write!(p, "{:?}", local_var_ty)?;
                                        if let Some(debug_var_name) =
                                            function.debug_var_names.get(&local_var_id)
                                        {
                                            write!(p, " /* {:?} */", debug_var_name)?;
                                        }
                                        Ok(())
                                    },
                                )
                            })?;
                            p.add_field("stack_space", |p| write!(p, "{}", stack_space))?;
                            p.add_field("body", |p| {
                                write!(p, "{}", body)?;
                                pretty_print_seq(p, self, function, body.body)
                            })
                        })
                    },
                )
            })
        });
        out
    }
}

fn pretty_print_seq<W: Write>(
    p: &mut PrettyPrinter<W>,
    program: &Program,
    function: &Function,
    insn_seq_id: InsnSeqId,
) -> fmt::Result {
    p.add_struct("", |p| {
        for (insn_idx, insn) in function.insn_seqs[insn_seq_id].iter_enumerated() {
            let val_names: Vec<_> = (0..)
                .map_while(|i| {
                    function.debug_val_names.get(&ValRef(InsnPc(insn_seq_id, insn_idx), i))
                })
                .collect();
            p.line()?;
            write!(p, "{}: {:?} = {}", insn_idx, val_names, insn)?;
            match insn {
                InsnKind::Block(Block { body, .. }) => {
                    pretty_print_seq(p, program, function, *body)?
                }
                InsnKind::IfElse(IfElse { then_body, else_body, .. }) => {
                    pretty_print_seq(p, program, function, *then_body)?;
                    pretty_print_seq(p, program, function, *else_body)?;
                }
                InsnKind::Loop(Loop { body, .. }) => pretty_print_seq(p, program, function, *body)?,
                InsnKind::VarLoad { var_id } | InsnKind::VarStore { var_id, .. } => {
                    if let Some(debug_var_name) = function.debug_var_names.get(var_id) {
                        write!(p, " /* {:?} */", debug_var_name)?;
                    }
                }
                InsnKind::CallUserFunction { function: fn_id, .. } => {
                    if let Some(debug_name) = &program.user_functions[*fn_id].debug_fn_name {
                        write!(p, " /* {:?} */", debug_name)?;
                    }
                }
                InsnKind::UserFunctionPtr { function: fn_id } => {
                    if let Some(debug_name) = &program.user_functions[*fn_id].debug_fn_name {
                        write!(p, " /* {:?} */", debug_name)?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    })
}

impl Add<usize> for InsnIdx {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        InsnIdx(self.0 + rhs)
    }
}
