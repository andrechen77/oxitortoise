use std::collections::HashMap;

use lir::smallvec::{self, SmallVec, ToSmallVec, smallvec};
use lir::typed_index_collections::ti_vec;
use lir::{Block, Function, IfElse, InsnKind, InsnSeqId, Loop, MultiValInsn, Reg, SingleValInsn};

use crate::stackify_generic::{self, InsnSeqStackification, StackManipulators};

/// Factors out all elements that are equal at the beginning of all the
/// sequences, and returns them in a vector in the same order.
fn factor_common_prefix<T: PartialEq, const N: usize>(mut sequences: [&mut Vec<T>; N]) -> Vec<T> {
    if N == 0 {
        return Vec::new();
    }
    if N == 1 {
        return sequences[0].drain(..).collect();
    }

    let mut result = Vec::new();
    let m = sequences[0].len();
    for _ in 0..m {
        let v = sequences[0].first().unwrap();
        if !sequences[1..].iter().all(|seq| seq.first() == Some(v)) {
            return result;
        }

        // all elements match, so add them to the result
        result.push(sequences[0].remove(0));
        for seq in &mut sequences[1..] {
            seq.remove(0);
        }
    }
    result
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Val {
    Reg {
        /// The register whose value we want to use.
        reg: Reg,
        /// A monotonically increasing index that differentiates between
        /// different values of the same register if the register was modified.
        nonce: u32,
    },
    /// The stack pointer of the current function. This cannot change.
    StackPtr,
}

#[derive(Default)]
struct ValTracker {
    /// Maps each register to the nonce of the last value of the register that
    /// was set. If a register does not exist in this map, its value was never
    /// set.
    current_reg_nonces: HashMap<Reg, u32>,
}

impl ValTracker {
    fn set_reg(&mut self, reg: Reg) -> Val {
        let nonce = self.current_reg_nonces.entry(reg).and_modify(|nonce| *nonce += 1).or_insert(0);
        Val::Reg { reg, nonce: *nonce }
    }

    /// Panics if the register has never been set.
    fn get_reg(&self, reg: Reg) -> Val {
        Val::Reg { reg, nonce: self.current_reg_nonces[&reg] }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CfgStackification {
    /// Maps each instruction sequence to the set of manipulators to be injected
    /// to get proper stack machine execution.
    pub seqs: HashMap<InsnSeqId, InsnSeqStackification<Val, usize>>,
}

pub fn stackify_cfg(function: &Function) -> CfgStackification {
    let mut vals = ValTracker::default();
    let mut result = CfgStackification { seqs: HashMap::new() };
    for (insn_seq_id, insn_seq) in function.insn_seqs.iter_enumerated() {
        let stackification = stackify_insn_seq(insn_seq, &mut vals);
        result.seqs.insert(insn_seq_id, stackification);
    }

    result
}

fn stackify_insn_seq(
    insn_seq: &[InsnKind],
    vals: &mut ValTracker,
) -> InsnSeqStackification<Val, usize> {
    // start the sequence with a fresh operand stack and no manipulators
    let mut op_stack = Vec::new();
    let mut getters = HashMap::new();

    let mut output = InsnSeqStackification {
        inputs: vec![],
        manips: ti_vec![StackManipulators {
            captures: 0,
            getters: vec![],
        }; insn_seq.len() + 1],
    };

    for (insn_idx, insn) in insn_seq.iter().enumerate() {
        let (inputs, outputs) = to_inputs_and_outputs(vals, insn);
        stackify_generic::stackify_single(
            &mut op_stack,
            &mut getters,
            &mut output.manips,
            insn_idx,
            inputs,
            outputs,
        );
    }

    // any excess operands on the stack should be eliminated
    stackify_generic::remove_excess_operands(
        op_stack.drain(..).zip(std::iter::repeat(false)),
        &mut output.manips,
        insn_seq.len(),
    );

    output
}

fn to_inputs_and_outputs(
    v: &mut ValTracker,
    insn: &InsnKind,
) -> (SmallVec<[Val; 2]>, SmallVec<[Val; 1]>) {
    match insn {
        InsnKind::SingleVal { out, insn } => {
            let inputs = match insn {
                SingleValInsn::Const { val: _ } => smallvec![],
                SingleValInsn::DeriveField { offset: _, ptr } => smallvec!(v.get_reg(*ptr)),
                SingleValInsn::DeriveElement { element_size: _, ptr, index } => {
                    smallvec!(v.get_reg(*ptr), v.get_reg(*index))
                }
                SingleValInsn::UserFunctionPtr { function: _ } => smallvec![],
                SingleValInsn::MemLoad { r#type: _, offset: _, ptr } => smallvec!(v.get_reg(*ptr)),
                SingleValInsn::StackLoad { r#type: _, offset: _ } => smallvec![],
                SingleValInsn::StackAddr { offset: _ } => smallvec![],
                SingleValInsn::UnaryOp { op: _, operand } => smallvec!(v.get_reg(*operand)),
                SingleValInsn::BinaryOp { op: _, lhs, rhs } => {
                    smallvec!(v.get_reg(*lhs), v.get_reg(*rhs))
                }
            };
            let output = v.set_reg(*out);
            (inputs, smallvec![output])
        }
        InsnKind::MultiVal { out, insn } => {
            let inputs = match insn {
                MultiValInsn::CallHostFunction { function: _, args }
                | MultiValInsn::CallUserFunction { function: _, args }
                | MultiValInsn::CallIndirectFunction { function: _, args } => {
                    args.iter().map(|reg| v.get_reg(*reg)).collect()
                }
            };
            let outputs = out.iter().map(|reg| v.set_reg(*reg)).collect();
            (inputs, outputs)
        }
        InsnKind::MemStore { r#type: _, offset: _, ptr, value } => {
            let inputs = smallvec![v.get_reg(*ptr), v.get_reg(*value)];
            let outputs = smallvec![];
            (inputs, outputs)
        }
        InsnKind::StackStore { r#type: _, offset: _, value } => {
            let inputs = smallvec![Val::StackPtr, v.get_reg(*value)];
            let outputs = smallvec![];
            (inputs, outputs)
        }
        InsnKind::Break { target: _ } => (smallvec![], smallvec![]),
        InsnKind::ConditionalBreak { target: _, condition } => {
            let inputs = smallvec![v.get_reg(*condition)];
            let outputs = smallvec![];
            (inputs, outputs)
        }
        // For these control flow instructions, we used to have an algorithm
        // that turned leading getters of the block into inputs to the block,
        // and any operands left on the stack when the block ended would be
        // outputs of the block. This required stackifying the inner instruction
        // sequence too, requiring mutual recursion between this function and
        // the `stackify_insn_seq` function. The resulting code was clever but
        // not super necessary, so this comment is what remains of it.
        InsnKind::Block(_block) => (smallvec![], smallvec![]),
        InsnKind::IfElse(_if_else) => (smallvec![], smallvec![]),
        InsnKind::Loop(_loop) => (smallvec![], smallvec![]),
    }
}

// /// Within an entire function, counts the number of getters exist for each
// /// value.
// pub fn count_getters(stk: &CfgStackification) -> HashMap<ValRef, usize> {
//     let mut result = HashMap::new();
//     for seq in stk.seqs.values() {
//         for manips in &seq.manips {
//             for v in &manips.getters {
//                 if let ValRefOrStackPtr::ValRef(v) = v {
//                     *result.entry(*v).or_insert(0) += 1;
//                 }
//             }
//         }
//     }
//     result
// }

/*
#[cfg(test)]
mod tests {
    use lir::lir_function;

    use super::*;
    use crate::stackify_generic::stackification;

    #[test]
    fn basic_sequence() {
        lir_function! {
            fn block() -> [I32],
            vars: [],
            stack_space: 0,
            main: {
                [a] = constant(I32, 0);
                [b] = constant(I32, 1);
                [c] = IAdd(a, b);
                break_(main)(c);
            }
        }

        let stackification = stackify_cfg(&block);
        assert_eq!(
            stackification.seqs[&InsnSeqId(0)],
            stackification! {
                inputs [];
                [InsnIdx(0)] cap(0) get[] [] => [a];
                [InsnIdx(1)] cap(0) get[] [] => [b];
                [InsnIdx(2)] cap(0) get[] [a, b] => [c];
                [InsnIdx(3)] cap(0) get[] [c] => [];
                [InsnIdx(4)] cap(0) get[] [] => [];
            }
        );
    }

    #[test]
    fn block_parameters() {
        // all leading getters in a block should be factored out and used as
        // inputs to the block instruction
        lir_function! {
            fn block() -> [I32],
            vars: [],
            stack_space: 0,
            main: {
                [a] = constant(I32, 10);
                [b] = constant(I32, 20);
                [outer] = block(-> [I32]) outer_block: {
                    [c] = IAdd(a, b);
                    [inner] = block(-> [I32]) inner_block: {
                        [d] = IAdd(c, a);
                        break_(inner_block)(d);
                    };
                    break_(outer_block)(inner);
                };
                break_(main)(outer);
            }
        }

        let stackification = stackify_cfg(&block);

        assert_eq!(
            stackification.seqs,
            HashMap::from_iter([
                (
                    InsnSeqId(0),
                    stackification! {
                        inputs [];
                        [InsnIdx(0)] cap(0) get[] [] => [ValRefOrStackPtr::ValRef(a)];
                        [InsnIdx(1)] cap(0) get[] [] => [ValRefOrStackPtr::ValRef(b)];
                        [InsnIdx(2)] cap(0) get[] [ValRefOrStackPtr::ValRef(a), ValRefOrStackPtr::ValRef(b)] => [ValRefOrStackPtr::ValRef(outer)];
                        [InsnIdx(3)] cap(0) get[] [ValRefOrStackPtr::ValRef(outer)] => [];
                        [InsnIdx(4)] cap(0) get[] [] => [];
                    },
                ),
                (
                    InsnSeqId(1),
                    stackification! {
                        inputs [ValRefOrStackPtr::ValRef(a), ValRefOrStackPtr::ValRef(b)];
                        [InsnIdx(0)] cap(0) get[] [ValRefOrStackPtr::ValRef(a), ValRefOrStackPtr::ValRef(b)] => [ValRefOrStackPtr::ValRef(c)];
                        [InsnIdx(1)] cap(0) get[ValRefOrStackPtr::ValRef(a)] [ValRefOrStackPtr::ValRef(c), ValRefOrStackPtr::ValRef(a)] => [ValRefOrStackPtr::ValRef(inner)];
                        [InsnIdx(2)] cap(0) get[] [ValRefOrStackPtr::ValRef(inner)] => [];
                        [InsnIdx(3)] cap(0) get[] [] => [];
                    },
                ),
                (
                    InsnSeqId(2),
                    stackification! {
                        inputs [ValRefOrStackPtr::ValRef(c), ValRefOrStackPtr::ValRef(a)];
                        [InsnIdx(0)] cap(0) get[] [ValRefOrStackPtr::ValRef(c), ValRefOrStackPtr::ValRef(a)] => [ValRefOrStackPtr::ValRef(d)];
                        [InsnIdx(1)] cap(0) get[] [ValRefOrStackPtr::ValRef(d)] => [];
                        [InsnIdx(2)] cap(0) get[] [] => [];
                    },
                ),
            ]),
        );
    }

    #[test]
    fn includes_branches() {
        lir_function! {
            fn block(I32 arg) -> [I32],
            vars: [],
            stack_space: 0,
            main: {
                [a] = constant(I32, 10);
                [b] = constant(I32, 20);
                [c] = constant(I32, 30);
                [d] = constant(I32, 40);
                [arg_val] = var_load(arg);
                [branch] = if_else(-> [I32])(arg_val) then: {
                    // get a, b, c
                    [res_0] = IAdd(a, IAdd(b, c));
                    break_(then)(res_0);
                } else_: {
                    // get a, b, d
                    [res_1] = ISub(a, ISub(b, d));
                    break_(else_)(res_1);
                };
                break_(main)(branch);
            }
        }

        let stackification = stackify_cfg(&block);

        assert_eq!(
            stackification.seqs,
            HashMap::from_iter([
                (
                    InsnSeqId(0),
                    stackification! {
                        inputs [];
                        [InsnIdx(0)] cap(0) get[] [] => [ValRefOrStackPtr::ValRef(a)];
                        [InsnIdx(1)] cap(0) get[] [] => [ValRefOrStackPtr::ValRef(b)];
                        [InsnIdx(2)] cap(0) get[] [] => [ValRefOrStackPtr::ValRef(c)];
                        [InsnIdx(3)] cap(1) get[] [] => [ValRefOrStackPtr::ValRef(d)];
                        [InsnIdx(4)] cap(1) get[] [] => [ValRefOrStackPtr::ValRef(arg_val)];
                        [InsnIdx(5)] cap(0) get[] [ValRefOrStackPtr::ValRef(a), ValRefOrStackPtr::ValRef(b), arg_val] => [ValRefOrStackPtr::ValRef(branch)];
                        [InsnIdx(6)] cap(0) get[] [ValRefOrStackPtr::ValRef(branch)] => [];
                        [InsnIdx(7)] cap(0) get[] [] => [];
                    },
                ),
                (
                    InsnSeqId(1),
                    // then branch
                    stackification! {
                        inputs [ValRefOrStackPtr::ValRef(a), ValRefOrStackPtr::ValRef(b)];
                        [InsnIdx(0)] cap(0) get[ValRefOrStackPtr::ValRef(c)] [ValRefOrStackPtr::ValRef(b), ValRefOrStackPtr::ValRef(c)] => [ValRefOrStackPtr::ValRef(ValRef(InsnPc(InsnSeqId(1), InsnIdx(0)), 0))];
                        [InsnIdx(1)] cap(0) get[] [ValRefOrStackPtr::ValRef(a), ValRefOrStackPtr::ValRef(ValRef(InsnPc(InsnSeqId(1), InsnIdx(0)), 0))] => [ValRefOrStackPtr::ValRef(res_0)];
                        [InsnIdx(2)] cap(0) get[] [ValRefOrStackPtr::ValRef(res_0)] => [];
                        [InsnIdx(3)] cap(0) get[] [] => [];
                    },
                ),
                (
                    InsnSeqId(2),
                    // else branch
                    stackification! {
                        inputs [ValRefOrStackPtr::ValRef(a), ValRefOrStackPtr::ValRef(b)];
                        [InsnIdx(0)] cap(0) get[ValRefOrStackPtr::ValRef(d)] [ValRefOrStackPtr::ValRef(b), ValRefOrStackPtr::ValRef(d)] => [ValRefOrStackPtr::ValRef(ValRef(InsnPc(InsnSeqId(2), InsnIdx(0)), 0))];
                        [InsnIdx(1)] cap(0) get[] [ValRefOrStackPtr::ValRef(a), ValRefOrStackPtr::ValRef(ValRef(InsnPc(InsnSeqId(2), InsnIdx(0)), 0))] => [ValRefOrStackPtr::ValRef(res_1)];
                        [InsnIdx(2)] cap(0) get[] [ValRefOrStackPtr::ValRef(res_1)] => [];
                        [InsnIdx(3)] cap(0) get[] [] => [];
                    },
                ),
            ]),
        );
    }
}
*/
