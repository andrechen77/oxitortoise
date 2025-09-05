use std::fmt::Display;

use super::*;

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Program { entrypoints, user_functions, host_functions } = self;
        f.debug_struct("Program")
            .field("entrypoints", entrypoints)
            .field("host_functions", host_functions)
            .field_with("user_functions", |f| {
                let mut list = f.debug_map();
                for (id, func) in user_functions {
                    list.key(&id);
                    list.value_with(|f| <Function as Display>::fmt(func, f));
                }
                list.finish()
            })
            .finish()
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Function {
            local_vars,
            num_parameters,
            stack_space,
            body,
            insn_seqs,
            debug_fn_name,
            debug_val_names,
            debug_var_names,
        } = self;
        f.debug_struct("Function")
            .field("local_vars", local_vars)
            .field("num_parameters", num_parameters)
            .field("stack_space", stack_space)
            .field("body", body)
            .field_with("insn_seqs", |f| {
                let mut map = f.debug_map();
                for (insn_seq_id, insn_seq) in insn_seqs.iter_enumerated() {
                    map.key(&insn_seq_id);
                    map.value_with(|f| {
                        let mut map = f.debug_map();
                        for (insn_idx, insn) in insn_seq.iter_enumerated() {
                            map.key(&insn_idx);
                            map.value_with(|f| <InsnKind as Display>::fmt(insn, f));
                        }
                        map.finish()
                    });
                }
                map.finish()
            })
            .field("debug_fn_name", debug_fn_name)
            .field("debug_val_names", debug_val_names)
            .field("debug_var_names", debug_var_names)
            .finish()
    }
}

impl Step for InsnIdx {
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        (end.0 - start.0, Some(end.0 - start.0))
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        Some(InsnIdx(start.0.checked_add(count)?))
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        Some(InsnIdx(start.0.checked_sub(count)?))
    }
}
