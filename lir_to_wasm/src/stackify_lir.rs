use std::collections::HashMap;

use lir::smallvec::{SmallVec, ToSmallVec, smallvec};
use lir::typed_index_collections::{TiVec, ti_vec};
use lir::{Block, Function, IfElse, InsnIdx, InsnKind, InsnPc, InsnSeqId, Loop, ValRef};

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
pub enum ValRefOrStackPtr {
    ValRef(ValRef),
    StackPtr,
}

impl From<ValRef> for ValRefOrStackPtr {
    fn from(v: ValRef) -> Self {
        ValRefOrStackPtr::ValRef(v)
    }
}

impl ValRefOrStackPtr {
    pub fn unwrap_val_ref(self) -> ValRef {
        match self {
            ValRefOrStackPtr::ValRef(v) => v,
            ValRefOrStackPtr::StackPtr => panic!("expected a val ref, but got a stack ptr"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CfgStackification {
    /// Maps each instruction sequence to the set of manipulators to be injected
    /// to get proper stack machine execution.
    pub seqs: TiVec<InsnSeqId, InsnSeqStackification<ValRefOrStackPtr, InsnIdx>>,
}

pub fn stackify_cfg(function: &Function) -> CfgStackification {
    let mut result = CfgStackification { seqs: TiVec::new() };
    stackify_insn_seq(function, function.body.body, &mut result.seqs);
    fn stackify_insn_seq(
        function: &Function,
        seq_id: InsnSeqId,
        out: &mut TiVec<InsnSeqId, InsnSeqStackification<ValRefOrStackPtr, InsnIdx>>,
    ) {
        // start the sequence with a fresh operand stack and no manipulators
        let mut op_stack = Vec::new();
        let mut getters = HashMap::new();

        let insns = &function.insn_seqs[seq_id];

        if seq_id.0 >= out.len() {
            assert_eq!(out.next_key(), seq_id);
            out.push(InsnSeqStackification {
                inputs: vec![],
                manips: ti_vec![StackManipulators {
                    captures: 0,
                    getters: vec![],
                    inputs: smallvec![],
                    outputs: smallvec![],
                }; insns.len() + 1],
            });
        }

        for (insn_idx, insn) in insns.iter_enumerated() {
            let pc = InsnPc(seq_id, insn_idx);
            let (inputs, outputs): (
                SmallVec<[ValRefOrStackPtr; 2]>,
                SmallVec<[ValRefOrStackPtr; 1]>,
            ) = match insn {
                InsnKind::FunctionArgs { .. } => {
                    // this doesn't actually output onto the stack. this is
                    // because arguments are just symbolic and don't translate
                    // to Wasm instructions
                    (smallvec![], smallvec![])
                }
                InsnKind::LoopArg { .. } => {
                    // this doesn't actually output onto the stack. this is
                    // because arguments are just symbolic and don't translate
                    // to Wasm instructions
                    (smallvec![], smallvec![])
                }
                InsnKind::Const { .. } => (smallvec![], smallvec![ValRef(pc, 0).into()]),
                InsnKind::UserFunctionPtr { .. } => (smallvec![], smallvec![ValRef(pc, 0).into()]),
                InsnKind::DeriveField { ptr, .. } => {
                    (smallvec![(*ptr).into()], smallvec![ValRef(pc, 0).into()])
                }
                InsnKind::DeriveElement { ptr, index, .. } => {
                    (smallvec![(*ptr).into(), (*index).into()], smallvec![ValRef(pc, 0).into()])
                }
                InsnKind::MemLoad { ptr, .. } => {
                    (smallvec![(*ptr).into()], smallvec![ValRef(pc, 0).into()])
                }
                InsnKind::MemStore { ptr, value, .. } => {
                    (smallvec![(*ptr).into(), (*value).into()], smallvec![])
                }
                InsnKind::StackLoad { .. } => (smallvec![], smallvec![ValRef(pc, 0).into()]),
                InsnKind::StackStore { value, .. } => {
                    (smallvec![ValRefOrStackPtr::StackPtr, (*value).into()], smallvec![])
                }
                InsnKind::StackAddr { .. } => (smallvec![], smallvec![ValRef(pc, 0).into()]),
                InsnKind::CallImportedFunction { args, output_type, .. } => (
                    args.iter().map(|v| (*v).into()).collect(),
                    (0..output_type.len())
                        .map(|i| ValRef(pc, i.try_into().unwrap()).into())
                        .collect(),
                ),
                InsnKind::CallUserFunction { args, output_type, .. } => (
                    args.iter().map(|v| (*v).into()).collect(),
                    (0..output_type.len())
                        .map(|i| ValRef(pc, i.try_into().unwrap()).into())
                        .collect(),
                ),
                InsnKind::UnaryOp { operand, .. } => {
                    (smallvec![(*operand).into()], smallvec![ValRef(pc, 0).into()])
                }
                InsnKind::BinaryOp { lhs, rhs, .. } => {
                    (smallvec![(*lhs).into(), (*rhs).into()], smallvec![ValRef(pc, 0).into()])
                }
                InsnKind::Break { values, .. } => {
                    (values.iter().map(|v| (*v).into()).collect(), smallvec![])
                }
                InsnKind::ConditionalBreak { condition, values, .. } => {
                    let outputs: SmallVec<[ValRefOrStackPtr; 1]> =
                        values.iter().map(|v| (*v).into()).collect();
                    let mut inputs = outputs.to_smallvec();
                    inputs.push((*condition).into());
                    (inputs, outputs)
                }
                InsnKind::Block(Block { body, output_type }) => {
                    // generate val refs for the outputs
                    let outputs = (0..output_type.len())
                        .map(|i| ValRef(pc, i.try_into().unwrap()).into())
                        .collect();

                    // stackify the inner block
                    stackify_insn_seq(function, *body, out);

                    // turn all leading getters into inputs to the block
                    let leading_manips = &mut out[*body].manips[InsnIdx(0)];
                    assert_eq!(
                        leading_manips.captures, 0,
                        "an insn seq stackified without parameters should not have any leading captures"
                    );
                    let leading_getters = std::mem::take(&mut leading_manips.getters);
                    let inputs = leading_getters.to_smallvec();
                    out[*body].inputs = leading_getters;

                    (inputs, outputs)
                }
                InsnKind::IfElse(IfElse { condition, output_type, then_body, else_body }) => {
                    // generate val refs for the outputs
                    let outputs = (0..output_type.len())
                        .map(|i| ValRef(pc, i.try_into().unwrap()).into())
                        .collect();

                    // stackify the inner blocks
                    stackify_insn_seq(function, *then_body, out);
                    stackify_insn_seq(function, *else_body, out);

                    // turn common leading getters into inputs to the if-else
                    let then_leading_manips = &mut out[*then_body].manips[InsnIdx(0)];
                    assert_eq!(then_leading_manips.captures, 0);
                    let mut then_leading_getters = std::mem::take(&mut then_leading_manips.getters);
                    let else_leading_manips = &mut out[*else_body].manips[InsnIdx(0)];
                    assert_eq!(else_leading_manips.captures, 0);
                    let mut else_leading_getters = std::mem::take(&mut else_leading_manips.getters);
                    let common_prefix = factor_common_prefix([
                        &mut then_leading_getters,
                        &mut else_leading_getters,
                    ]);
                    let mut inputs = common_prefix.to_smallvec();
                    inputs.push((*condition).into());
                    out[*then_body].inputs = common_prefix.clone();
                    out[*then_body].manips[InsnIdx(0)].getters = then_leading_getters;
                    out[*else_body].inputs = common_prefix;
                    out[*else_body].manips[InsnIdx(0)].getters = else_leading_getters;

                    (inputs, outputs)
                }
                InsnKind::Loop(Loop { inputs, output_type, body }) => {
                    // generate val refs for the outputs
                    let outputs = (0..output_type.len())
                        .map(|i| ValRef(pc, i.try_into().unwrap()).into())
                        .collect();

                    // stackify the inner block
                    stackify_insn_seq(function, *body, out);

                    let inputs = inputs.iter().map(|v| (*v).into()).collect();

                    (inputs, outputs)
                }
            };
            stackify_generic::stackify_single(
                &mut op_stack,
                &mut getters,
                &mut out[seq_id].manips,
                insn_idx,
                inputs,
                outputs,
            );
        }

        // any excess operands on the stack should be eliminated
        stackify_generic::remove_excess_operands(
            op_stack.drain(..).zip(std::iter::repeat(false)),
            &mut out[seq_id].manips,
            insns.next_key(),
        );
    }

    result
}

/// Within an entire function, counts the number of getters exist for each
/// value.
pub fn count_getters(stk: &CfgStackification) -> HashMap<ValRef, usize> {
    let mut result = HashMap::new();
    for seq in &stk.seqs {
        for manips in &seq.manips {
            for v in &manips.getters {
                if let ValRefOrStackPtr::ValRef(v) = v {
                    *result.entry(*v).or_insert(0) += 1;
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use lir::lir_function;
    use lir::typed_index_collections::ti_vec;

    use super::*;
    use crate::stackify_generic::stackification;

    #[test]
    fn basic_sequence() {
        lir_function! {
            fn block() -> (I32),
            stack_space: 0,
            main: {
                %a = constant(I32, 0);
                %b = constant(I32, 1);
                %c = IAdd(a, b);
                break_(main)(c);
            }
        }

        let stackification = stackify_cfg(&block);
        assert_eq!(
            stackification.seqs[InsnSeqId(0)],
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
            fn block() -> (I32),
            stack_space: 0,
            main: {
                %a = constant(I32, 10);
                %b = constant(I32, 20);
                %outer = block(-> (I32)) outer_block: {
                    %c = IAdd(a, b);
                    %inner = block(-> (I32)) inner_block: {
                        %d = IAdd(c, a);
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
            ti_vec![
                stackification! {
                    inputs [];
                    [InsnIdx(0)] cap(0) get[] [] => [ValRefOrStackPtr::ValRef(a)];
                    [InsnIdx(1)] cap(0) get[] [] => [ValRefOrStackPtr::ValRef(b)];
                    [InsnIdx(2)] cap(0) get[] [ValRefOrStackPtr::ValRef(a), ValRefOrStackPtr::ValRef(b)] => [ValRefOrStackPtr::ValRef(outer)];
                    [InsnIdx(3)] cap(0) get[] [ValRefOrStackPtr::ValRef(outer)] => [];
                    [InsnIdx(4)] cap(0) get[] [] => [];
                },
                stackification! {
                    inputs [ValRefOrStackPtr::ValRef(a), ValRefOrStackPtr::ValRef(b)];
                    [InsnIdx(0)] cap(0) get[] [ValRefOrStackPtr::ValRef(a), ValRefOrStackPtr::ValRef(b)] => [ValRefOrStackPtr::ValRef(c)];
                    [InsnIdx(1)] cap(0) get[ValRefOrStackPtr::ValRef(a)] [ValRefOrStackPtr::ValRef(c), ValRefOrStackPtr::ValRef(a)] => [ValRefOrStackPtr::ValRef(inner)];
                    [InsnIdx(2)] cap(0) get[] [ValRefOrStackPtr::ValRef(inner)] => [];
                    [InsnIdx(3)] cap(0) get[] [] => [];
                },
                stackification! {
                    inputs [ValRefOrStackPtr::ValRef(c), ValRefOrStackPtr::ValRef(a)];
                    [InsnIdx(0)] cap(0) get[] [ValRefOrStackPtr::ValRef(c), ValRefOrStackPtr::ValRef(a)] => [ValRefOrStackPtr::ValRef(d)];
                    [InsnIdx(1)] cap(0) get[] [ValRefOrStackPtr::ValRef(d)] => [];
                    [InsnIdx(2)] cap(0) get[] [] => [];
                },
            ],
        );
    }

    #[test]
    fn includes_branches() {
        lir_function! {
            fn block() -> (I32),
            stack_space: 0,
            main: {
                %arg = arguments(-> (I32));
                %a = constant(I32, 10);
                %b = constant(I32, 20);
                %c = constant(I32, 30);
                %d = constant(I32, 40);
                %branch = if_else(-> (I32))(arg) then: {
                    // get a, b, c
                    %res_0 = IAdd(a, IAdd(b, c));
                    break_(then)(res_0);
                } else_: {
                    // get a, b, d
                    %res_1 = ISub(a, ISub(b, d));
                    break_(else_)(res_1);
                };
                break_(main)(branch);
            }
        }

        let stackification = stackify_cfg(&block);

        assert_eq!(
            stackification.seqs,
            ti_vec![
                stackification! {
                    inputs [];
                    [InsnIdx(0)] cap(0) get[] [] => [];
                    [InsnIdx(1)] cap(0) get[] [] => [ValRefOrStackPtr::ValRef(a)];
                    [InsnIdx(2)] cap(0) get[] [] => [ValRefOrStackPtr::ValRef(b)];
                    [InsnIdx(3)] cap(0) get[ValRefOrStackPtr::ValRef(arg)] [] => [ValRefOrStackPtr::ValRef(c)];
                    [InsnIdx(4)] cap(1) get[] [] => [ValRefOrStackPtr::ValRef(d)];
                    [InsnIdx(5)] cap(1) get[] [ValRefOrStackPtr::ValRef(a), ValRefOrStackPtr::ValRef(b), arg] => [ValRefOrStackPtr::ValRef(branch)];
                    [InsnIdx(6)] cap(0) get[] [ValRefOrStackPtr::ValRef(branch)] => [];
                    [InsnIdx(7)] cap(0) get[] [] => [];
                },
                // then branch
                stackification! {
                    inputs [ValRefOrStackPtr::ValRef(a), ValRefOrStackPtr::ValRef(b)];
                    [InsnIdx(0)] cap(0) get[ValRefOrStackPtr::ValRef(c)] [ValRefOrStackPtr::ValRef(b), ValRefOrStackPtr::ValRef(c)] => [ValRefOrStackPtr::ValRef(ValRef(InsnPc(InsnSeqId(1), InsnIdx(0)), 0))];
                    [InsnIdx(1)] cap(0) get[] [ValRefOrStackPtr::ValRef(a), ValRefOrStackPtr::ValRef(ValRef(InsnPc(InsnSeqId(1), InsnIdx(0)), 0))] => [ValRefOrStackPtr::ValRef(res_0)];
                    [InsnIdx(2)] cap(0) get[] [ValRefOrStackPtr::ValRef(res_0)] => [];
                    [InsnIdx(3)] cap(0) get[] [] => [];
                },
                // else branch
                stackification! {
                    inputs [ValRefOrStackPtr::ValRef(a), ValRefOrStackPtr::ValRef(b)];
                    [InsnIdx(0)] cap(0) get[ValRefOrStackPtr::ValRef(d)] [ValRefOrStackPtr::ValRef(b), ValRefOrStackPtr::ValRef(d)] => [ValRefOrStackPtr::ValRef(ValRef(InsnPc(InsnSeqId(2), InsnIdx(0)), 0))];
                    [InsnIdx(1)] cap(0) get[] [ValRefOrStackPtr::ValRef(a), ValRefOrStackPtr::ValRef(ValRef(InsnPc(InsnSeqId(2), InsnIdx(0)), 0))] => [ValRefOrStackPtr::ValRef(res_1)];
                    [InsnIdx(2)] cap(0) get[] [ValRefOrStackPtr::ValRef(res_1)] => [];
                    [InsnIdx(3)] cap(0) get[] [] => [];
                },
            ]
        )
    }
}
