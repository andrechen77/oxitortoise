use std::{cell::Cell, collections::HashMap, iter::Step as _};

use smallvec::{SmallVec, ToSmallVec, smallvec};
use typed_index_collections::{TiVec, ti_vec};

use crate::{
    lir::{Block, Function, IfElse, InsnKind, InsnPc, InsnSeqId, Loop, ValRef},
    stackify::{self, InsnSeqStackification, StackManipulators},
};

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

#[derive(Debug, PartialEq, Eq)]
pub struct CfgStackification {
    /// Maps each instruction sequence to the set of manipulators to be injected
    /// to get proper stack machine execution.
    pub seqs: TiVec<InsnSeqId, InsnSeqStackification<ValRef, InsnPc>>,
}

pub fn stackify_cfg(function: &Function) -> CfgStackification {
    let mut result = CfgStackification { seqs: TiVec::new() };
    let current_val_ref = Cell::new(ValRef(0));
    stackify_insn_seq(function, function.body.body, &mut result.seqs, &current_val_ref);
    fn stackify_insn_seq(
        function: &Function,
        seq_id: InsnSeqId,
        out: &mut TiVec<InsnSeqId, InsnSeqStackification<ValRef, InsnPc>>,
        current_val_ref: &Cell<ValRef>,
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
        let next_val = || {
            let val_ref = current_val_ref.get();
            current_val_ref.set(ValRef::forward(val_ref, 1));
            val_ref
        };

        for (pc, insn) in insns.iter_enumerated() {
            let (inputs, outputs): (SmallVec<[ValRef; 2]>, SmallVec<[ValRef; 1]>) = match insn {
                InsnKind::Argument { .. } => {
                    // take up a val ref, but don't actually output it onto
                    // the stack. this is because arguments are just symbolic
                    // and don't translate to Wasm instructions
                    next_val();
                    (smallvec![], smallvec![])
                }
                InsnKind::LoopArgument { .. } => {
                    // take up a val ref, but don't actually output it onto
                    // the stack. this is because loop arguments are just
                    // symbolic and don't translate to Wasm instructions
                    next_val();
                    (smallvec![], smallvec![])
                }
                InsnKind::Const { .. } => (smallvec![], smallvec![next_val()]),
                InsnKind::DeriveField { ptr, .. } => (smallvec![*ptr], smallvec![next_val()]),
                InsnKind::DeriveElement { ptr, index, .. } => {
                    (smallvec![*ptr, *index], smallvec![next_val()])
                }
                InsnKind::MemLoad { ptr, .. } => (smallvec![*ptr], smallvec![next_val()]),
                InsnKind::MemStore { ptr, value, .. } => (smallvec![*ptr, *value], smallvec![]),
                InsnKind::StackLoad { .. } => (smallvec![], smallvec![next_val()]),
                InsnKind::StackStore { value, .. } => (smallvec![*value], smallvec![]),
                #[allow(unreachable_code)]
                InsnKind::CallImportedFunction { args, .. } => {
                    (args.iter().map(|v| *v).collect(), todo!("look up return type of function"))
                }
                #[allow(unreachable_code)]
                InsnKind::CallUserFunction { args, .. } => {
                    (args.iter().map(|v| *v).collect(), todo!("look up return type of function"))
                }
                InsnKind::UnaryOp { operand, .. } => (smallvec![*operand], smallvec![next_val()]),
                InsnKind::BinaryOp { lhs, rhs, .. } => {
                    (smallvec![*lhs, *rhs], smallvec![next_val()])
                }
                InsnKind::Break { values, .. } => {
                    (values.iter().map(|v| *v).collect(), smallvec![])
                }
                InsnKind::ConditionalBreak { condition, values, .. } => {
                    let outputs: SmallVec<[ValRef; 1]> = values.iter().map(|v| *v).collect();
                    let mut inputs = outputs.to_smallvec();
                    inputs.push(*condition);
                    (inputs, outputs)
                }
                InsnKind::Block(Block { body, output_type }) => {
                    // generate val refs for the outputs
                    let outputs = output_type.as_ref().iter().map(|_| next_val()).collect();

                    // stackify the inner block
                    stackify_insn_seq(function, *body, out, current_val_ref);

                    // turn all leading getters into inputs to the block
                    let leading_manips = &mut out[*body].manips[InsnPc(0)];
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
                    let outputs = output_type.as_ref().iter().map(|_| next_val()).collect();

                    // stackify the inner blocks
                    stackify_insn_seq(function, *then_body, out, current_val_ref);
                    stackify_insn_seq(function, *else_body, out, current_val_ref);

                    // turn common leading getters into inputs to the if-else
                    let then_leading_manips = &mut out[*then_body].manips[InsnPc(0)];
                    assert_eq!(then_leading_manips.captures, 0);
                    let mut then_leading_getters = std::mem::take(&mut then_leading_manips.getters);
                    let else_leading_manips = &mut out[*else_body].manips[InsnPc(0)];
                    assert_eq!(else_leading_manips.captures, 0);
                    let mut else_leading_getters = std::mem::take(&mut else_leading_manips.getters);
                    let common_prefix = factor_common_prefix([
                        &mut then_leading_getters,
                        &mut else_leading_getters,
                    ]);
                    let mut inputs = common_prefix.to_smallvec();
                    inputs.push(*condition);
                    out[*then_body].inputs = common_prefix.clone();
                    out[*then_body].manips[InsnPc(0)].getters = then_leading_getters;
                    out[*else_body].inputs = common_prefix;
                    out[*else_body].manips[InsnPc(0)].getters = else_leading_getters;

                    (inputs, outputs)
                }
                InsnKind::Loop(Loop { inputs, output_type, body }) => {
                    // generate val refs for the outputs
                    let outputs = output_type.as_ref().iter().map(|_| next_val()).collect();

                    // stackify the inner block
                    stackify_insn_seq(function, *body, out, current_val_ref);

                    let inputs = inputs.to_smallvec();

                    (inputs, outputs)
                }
            };
            stackify::stackify_single(
                &mut op_stack,
                &mut getters,
                &mut out[seq_id].manips,
                pc,
                inputs,
                outputs,
            );
        }

        // any excess operands on the stack should be eliminated
        stackify::remove_excess_operands(
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
                *result.entry(*v).or_insert(0) += 1;
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use typed_index_collections::ti_vec;

    use super::*;
    use crate::lir::lir_function;
    use crate::stackify::stackification;

    #[test]
    fn basic_sequence() {
        lir_function! {
            fn block() -> (I32) main: {
                %a = constant(I32, 0);
                %b = constant(I32, 1);
                %c = Add(a, b);
                break_(main)(c);
            }
        }

        let stackification = stackify_cfg(&block);
        assert_eq!(stackification.seqs[InsnSeqId(0)], stackification! { inputs []; });
    }

    #[test]
    fn block_parameters() {
        // all leading getters in a block should be factored out and used as
        // inputs to the block instruction
        lir_function! {
            fn block() -> (I32) main: {
                %a = constant(I32, 10);
                %b = constant(I32, 20);
                %outer = block(-> (I32)) outer_block: {
                    %c = Add(a, b);
                    %inner = block(-> (I32)) inner_block: {
                        %d = Add(c, a);
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
                    [InsnPc(0)] cap(0) get[] [] => [a];
                    [InsnPc(1)] cap(0) get[] [] => [b];
                    [InsnPc(2)] cap(0) get[] [] => [outer];
                    [InsnPc(3)] cap(0) get[] [outer] => [];
                    [InsnPc(4)] cap(0) get[] [] => [];
                },
                stackification! {
                    inputs [a, b];
                    [InsnPc(0)] cap(0) get[] [a, b] => [c];
                    [InsnPc(1)] cap(0) get[a] [c, a] => [inner];
                    [InsnPc(2)] cap(0) get[] [inner] => [];
                    [InsnPc(3)] cap(0) get[] [] => [];
                },
                stackification! {
                    inputs [c, a];
                    [InsnPc(0)] cap(0) get[] [c, a] => [d];
                    [InsnPc(1)] cap(0) get[] [d] => [];
                    [InsnPc(2)] cap(0) get[] [] => [];
                },
            ],
        );
    }

    #[test]
    fn includes_branches() {
        lir_function! {
            fn block() -> (I32) main: {
                %arg = argument(I32, 0);
                %a = constant(I32, 10);
                %b = constant(I32, 20);
                %c = constant(I32, 30);
                %d = constant(I32, 40);
                %branch = if_else(-> (I32))(arg) then: {
                    // get a, b, c
                    %res_0 = Add(a, Add(b, c));
                    break_(then)(res_0);
                } else_: {
                    // get a, b, d
                    %res_1 = Sub(a, Sub(b, d));
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
                    [InsnPc(0)] cap(0) get[] [] => [arg];
                    [InsnPc(1)] cap(1) get[] [] => [a];
                    [InsnPc(2)] cap(0) get[] [] => [b];
                    [InsnPc(3)] cap(0) get[arg] [] => [c];
                    [InsnPc(4)] cap(1) get[] [] => [d];
                    [InsnPc(5)] cap(1) get[] [a, b, arg] => [branch];
                    [InsnPc(6)] cap(0) get[] [branch] => [];
                    [InsnPc(7)] cap(0) get[] [] => [];
                },
                // then branch
                stackification! {
                    inputs [a, b];
                    [InsnPc(0)] cap(0) get[c] [b, c] => [ValRef::backward(res_0, 1)];
                    [InsnPc(1)] cap(0) get[] [a, ValRef::backward(res_0, 1)] => [res_0];
                    [InsnPc(2)] cap(0) get[] [res_0] => [];
                    [InsnPc(3)] cap(0) get[] [] => [];
                },
                // else branch
                stackification! {
                    inputs [a, b];
                    [InsnPc(0)] cap(0) get[d] [b, d] => [ValRef::backward(res_1, 1)];
                    [InsnPc(1)] cap(0) get[] [a, ValRef::backward(res_1, 1)] => [res_1];
                    [InsnPc(2)] cap(0) get[] [res_1] => [];
                    [InsnPc(3)] cap(0) get[] [] => [];
                },
            ]
        )
    }
}
