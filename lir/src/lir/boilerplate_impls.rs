use std::{fmt::Display, ops::Add};

use super::*;

// TODO for all display impls, use debug_closure_helpers once stabilized. see
// how the code looked in 4e8f50af940c6cacd4bb5511ad156093f0da4e7b

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Program { entrypoints, user_functions, host_functions } = self;
        write!(f, "Program {{ entrypoints: ")?;
        entrypoints.fmt(f)?;
        write!(f, ", host_functions: ")?;
        host_functions.fmt(f)?;
        write!(f, ", user_functions: {{")?;
        let mut iter = user_functions.iter();
        if let Some((id, func)) = iter.next() {
            write!(f, "{:?}: ", id)?;
            <Function as Display>::fmt(func, f)?;
            for (id, func) in iter {
                write!(f, ", {:?}: ", id)?;
                <Function as Display>::fmt(func, f)?;
            }
        }
        write!(f, "}} }}")
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
        write!(f, "Function {{ local_vars: ")?;
        local_vars.fmt(f)?;
        write!(f, ", num_parameters: ")?;
        write!(f, "{}", num_parameters)?;
        write!(f, ", stack_space: ")?;
        write!(f, "{}", stack_space)?;
        write!(f, ", body: ")?;
        write!(f, "{}", body)?;
        write!(f, ", insn_seqs: {{")?;
        let mut iter = insn_seqs.iter_enumerated();
        if let Some((insn_seq_id, insn_seq)) = iter.next() {
            write!(f, "{:?}: {{", insn_seq_id)?;
            let mut inner_iter = insn_seq.iter_enumerated();
            if let Some((insn_idx, insn)) = inner_iter.next() {
                write!(f, "{:?}: ", insn_idx)?;
                <InsnKind as Display>::fmt(insn, f)?;
                for (insn_idx, insn) in inner_iter {
                    write!(f, ", {:?}: ", insn_idx)?;
                    <InsnKind as Display>::fmt(insn, f)?;
                }
            }
            write!(f, "}}")?;
            for (insn_seq_id, insn_seq) in iter {
                write!(f, ", {:?}: {{", insn_seq_id)?;
                let mut inner_iter = insn_seq.iter_enumerated();
                if let Some((insn_idx, insn)) = inner_iter.next() {
                    write!(f, "{:?}: ", insn_idx)?;
                    <InsnKind as Display>::fmt(insn, f)?;
                    for (insn_idx, insn) in inner_iter {
                        write!(f, ", {:?}: ", insn_idx)?;
                        <InsnKind as Display>::fmt(insn, f)?;
                    }
                }
                write!(f, "}}")?;
            }
        }
        write!(f, "}}, debug_fn_name: ")?;
        debug_fn_name.fmt(f)?;
        write!(f, ", debug_val_names: ")?;
        debug_val_names.fmt(f)?;
        write!(f, ", debug_var_names: ")?;
        debug_var_names.fmt(f)?;
        write!(f, " }}")
    }
}

impl Add<usize> for InsnIdx {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        InsnIdx(self.0 + rhs)
    }
}
