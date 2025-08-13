use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    ops::Range,
};

use smallvec::{SmallVec, ToSmallVec, smallvec};
use typed_index_collections::TiVec;

use crate::{
    lir::{InsnKind, InsnPc, InsnRefIter},
    stackify::{self, InsnTreeInfo, OutputMode},
};

pub type Stackification = stackify::Stackification<InsnPc, TiVec<InsnPc, InsnTreeInfo<InsnPc>>>;

struct InsnsWithCmpdInputs<'a> {
    insns: &'a TiVec<InsnPc, InsnKind>,
    cmpd_inputs: &'a HashMap<InsnPc, Vec<InsnPc>>,
}

impl<'a> stackify::InsnUniverse for InsnsWithCmpdInputs<'a> {
    type Pc = InsnPc;

    fn instructions_in_range(&self, range: Range<Self::Pc>) -> impl Iterator<Item = Self::Pc> {
        InsnRefIter::new_with_range(self.insns, range).map(|(_, pc)| pc)
    }

    fn succ_of(&self, insn: Self::Pc) -> Self::Pc {
        self.insns[insn].extent(insn).1
    }

    fn inputs_of(&self, insn: Self::Pc) -> impl DoubleEndedIterator<Item = Self::Pc> {
        match &self.insns[insn] {
            InsnKind::Argument { .. } => smallvec![],
            InsnKind::Const { .. } => smallvec![],
            InsnKind::Project { multivalue, .. } => smallvec![*multivalue],
            InsnKind::DeriveField { ptr, .. } => smallvec![*ptr],
            InsnKind::DeriveElement { ptr, index, .. } => smallvec![*ptr, *index],
            InsnKind::MemLoad { ptr, .. } => smallvec![*ptr],
            InsnKind::MemStore { ptr, value, .. } => smallvec![*ptr, *value],
            InsnKind::StackLoad { .. } => smallvec![],
            InsnKind::StackStore { value, .. } => smallvec![*value],
            InsnKind::CallImportedFunction { args, .. } => args.to_smallvec(),
            InsnKind::CallUserFunction { args, .. } => args.to_smallvec(),
            InsnKind::UnaryOp { operand, .. } => smallvec![*operand],
            InsnKind::BinaryOp { lhs, rhs, .. } => smallvec![*lhs, *rhs],
            InsnKind::Break { values, .. } => values.to_smallvec(),
            InsnKind::ConditionalBreak { condition, values, .. } => {
                let mut inputs: SmallVec<[InsnPc; 2]> = values.to_smallvec();
                inputs.push(*condition);
                inputs
            }
            InsnKind::Block { .. } => {
                self.cmpd_inputs.get(&insn).map(|v| v.to_smallvec()).unwrap_or_default()
            }
            InsnKind::IfElse { condition, .. } => {
                let mut inputs =
                    self.cmpd_inputs.get(&insn).map(|v| v.to_smallvec()).unwrap_or_default();
                inputs.push(*condition);
                println!("inputs of block are {:?}", inputs);
                inputs
            }
            InsnKind::Loop { .. } => {
                self.cmpd_inputs.get(&insn).map(|v| v.to_smallvec()).unwrap_or_default()
            }
            InsnKind::LoopArgument { initial_value: _ } => smallvec![],
        }
        .into_iter()
    }
}

/// Factors out all elements that are equal at the beginning of all the
/// sequences, and returns them in a vector in the same order.
fn factor_common_prefix<T: PartialEq, const N: usize>(
    mut sequences: [&mut VecDeque<T>; N],
) -> Vec<T> {
    if N == 0 {
        return Vec::new();
    }
    if N == 1 {
        return sequences[0].drain(..).collect();
    }

    let mut result = Vec::new();
    let m = sequences[0].len();
    for _ in 0..m {
        let v = sequences[0].front().unwrap();
        if !sequences[1..].iter().all(|seq| seq.front() == Some(v)) {
            return result;
        }

        // all elements match, so add them to the result
        result.push(sequences[0].pop_front().unwrap());
        for seq in &mut sequences[1..] {
            seq.pop_front();
        }
    }
    result
}

pub fn stackify_lir(insns: &TiVec<InsnPc, InsnKind>) -> Stackification {
    let mut stackification = Stackification {
        forest: (0..insns.len())
            .map(|pc| InsnTreeInfo {
                output_mode: OutputMode::Available,
                subtree_start: InsnPc::from(pc),
            })
            .collect(),
        getters: BTreeMap::new(),
    };

    // stores inputs that for compound instructions that might take parameters
    let mut cmpd_inputs = HashMap::new();

    // recursive through the compound instructions. stackify the deeper
    // compound instructions first,
    stackify_recursive(
        insns,
        InsnPc::from(0)..InsnPc::from(insns.len()),
        &mut stackification,
        &mut cmpd_inputs,
    );

    // stackify the instructions in the range, recursively stackifying all
    // inner compound instructions as well
    fn stackify_recursive(
        insns: &TiVec<InsnPc, InsnKind>,
        range: Range<InsnPc>,
        stackification: &mut Stackification,
        cmpd_inputs: &mut HashMap<InsnPc, Vec<InsnPc>>,
    ) {
        // stackify all inner instructions first
        for (inner_seqs, pc) in InsnRefIter::new_with_range(insns, range.clone()) {
            for inner_seq in &inner_seqs {
                stackify_recursive(insns, inner_seq.clone(), stackification, cmpd_inputs);
            }

            println!(
                "encountered compound instruction at {:?}, cmpd_inputs: {:?}",
                pc, cmpd_inputs
            );

            // leading getters in the inner sequences can be factored out
            // and turned into inputs of the compound instruction
            match &inner_seqs[..] {
                [] => {}
                [seq] => {
                    // there is only one inner sequences, so we can "factor out"
                    // all parameters and use them as inputs to the compound
                    // instruction
                    let params = stackification.getters.remove(&seq.start).unwrap_or_default();
                    cmpd_inputs.entry(pc).or_default().extend(params);
                }
                [a, b] => 'factoring: {
                    println!("factoring inner sequences of {:?}", pc);
                    let Some(mut inner_0) = stackification.getters.remove(&a.start) else {
                        break 'factoring;
                    };
                    let Some(mut inner_1) = stackification.getters.remove(&b.start) else {
                        break 'factoring;
                    };
                    let common_prefix = factor_common_prefix([&mut inner_0, &mut inner_1]);
                    let prev_entry = cmpd_inputs.insert(pc, common_prefix);
                    assert!(
                        prev_entry.is_none(),
                        "there should be no previous entry for this compound insn"
                    );
                    // Only insert the getters back if the list is non-empty, to
                    // make testing easier
                    if !inner_0.is_empty() {
                        stackification.getters.insert(a.start, inner_0);
                    }
                    if !inner_1.is_empty() {
                        stackification.getters.insert(b.start, inner_1);
                    }
                }
                _ => unimplemented!(
                    "did not expect a compound instruction with more than 2 inner sequences"
                ),
            }
        }

        // split the range by conditional branches, since in Wasm, the
        // conditional branch spits its arguments back onto the stack if the
        // branch is not taken, but the stackify function does not have a way
        // to handle this
        let mut split_start = range.start;
        let mut split_end = range.start;
        while split_end < range.end {
            let this_insn = &insns[split_end];
            split_end = insns[split_end].extent(split_end).1; // successor
            if matches!(this_insn, InsnKind::ConditionalBreak { .. }) || split_end == range.end {
                stackify::stackify_sequential(
                    &InsnsWithCmpdInputs { insns, cmpd_inputs },
                    split_start..split_end,
                    stackification,
                );
                split_start = split_end;
            }
        }
    }

    stackification
}

fn print_with_stackification(insns: &TiVec<InsnPc, InsnKind>, stackification: &Stackification) {
    for pc in (0..insns.len()).map(InsnPc::from) {
        // look for getters
        if let Some(values) = stackification.getters.get(&pc) {
            for value in values {
                println!("get ${}", usize::from(*value));
            }
        }

        let output_mode = match stackification.forest[pc].output_mode {
            OutputMode::Available => "A".to_string(),
            OutputMode::Release { parent } => format!("R{}", usize::from(parent)),
            OutputMode::Capture { parent } => format!("C{}", usize::from(parent)),
        };

        println!("{} ${} = {:?}", output_mode, usize::from(pc), insns[pc],);
    }
}

#[cfg(test)]
mod tests {
    use typed_index_collections::ti_vec;

    use super::*;
    use crate::{
        lir::{BinaryOpcode, InsnKind::*, ValType, instructions},
        stackification,
    };

    #[test]
    fn basic_sequence() {
        instructions! {
            let insns;
            a = [constant(I32, 0)];
            b = [constant(I32, 1)];
            c = [add(a, b)];
        }

        let stackification = stackify_lir(&insns);
        assert_eq!(stackification, stackification! {
            Stackification;
            InsnPc;
            count 3;

            [(a) =| c]
            [(b) =| c]
            [(c) ==]
        });
    }

    #[test]
    fn block_parameters() {
        // all leading getters in a block should be factored out and used as
        // inputs to the block instruction
        instructions! {
            let insns;
            a = [constant(I32, 10)];
            b = [constant(I32, 20)];
            outer = [block {
                c = [add(a, b)];
                inner = [block {
                    d = [add(c, a)];
                    break_2 = [break_(0)(d)];
                }];
                break_1 = [break_(0)(inner)];
            }];
            break_0 = [break_(0)(outer)];
        }

        let stackification = stackify_lir(&insns);

        assert_eq!(stackification, stackification! {
            Stackification;
            InsnPc;
            count 9;

            [(a) =| outer]
            [(b) =| outer]
            [(outer) =| break_0]
            [(c) =| inner]
            [(inner) <~ a]
            [(inner) =| break_1]
            [(d) =| break_2]
            [(break_2) ==]
            [(break_1) ==]
            [(break_0) ==]
        })
    }

    #[test]
    fn includes_branches() {
        instructions! {
            let insns;
            arg = [argument(I32, 0)];
            a = [constant(I32, 10)];
            b = [constant(I32, 20)];
            branch = [if_else(arg) {
                d_0 = [add(a, b)];
                break_0 = [break_(0)(d_0)];
            } {
                d_1 = [sub(a, b)];
                break_1 = [break_(0)(d_1)];
            }];
            break_2 = [break_(0)(branch)];
        }

        let stackification = stackify_lir(&insns);

        assert_eq!(stackification, stackification! {
            Stackification;
            InsnPc;
            count 9;

            [(arg) ==]
            [(a) =| branch]
            [(b) =| branch]
            [(branch) <~ arg]
            [(branch) =| break_2]
            [(d_0) =| break_0]
            [(break_0) ==]
            [(d_1) =| break_1]
            [(break_1) ==]
            [(break_2) ==]

        })
    }
}
