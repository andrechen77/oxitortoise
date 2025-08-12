use std::{
    collections::{BTreeMap, HashMap},
    ops::Range,
};

use smallvec::{SmallVec, ToSmallVec, smallvec};
use typed_index_collections::TiVec;

use crate::{
    lir::{InsnKind, InsnPc, InsnRefIter},
    stackify::{self, InsnTreeInfo, OutputMode},
};

type Stackification = stackify::Stackification<InsnPc, TiVec<InsnPc, InsnTreeInfo<InsnPc>>>;

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
            InsnKind::ConditionalBreak {
                condition, values, ..
            } => {
                let mut inputs: SmallVec<[InsnPc; 2]> = values.to_smallvec();
                inputs.push(*condition);
                inputs
            }
            InsnKind::Block { .. } => self
                .cmpd_inputs
                .get(&insn)
                .map(|v| v.to_smallvec())
                .unwrap_or_default(),
            InsnKind::IfElse { condition, .. } => {
                let mut inputs = self
                    .cmpd_inputs
                    .get(&insn)
                    .map(|v| v.to_smallvec())
                    .unwrap_or_default();
                inputs.push(*condition);
                inputs
            }
            InsnKind::Loop { .. } => self
                .cmpd_inputs
                .get(&insn)
                .map(|v| v.to_smallvec())
                .unwrap_or_default(),
            InsnKind::LoopArgument { initial_value: _ } => smallvec![],
        }
        .into_iter()
    }
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
            // leading getters in the inner sequences can be factored out
            // and turned into inputs of the compound instruction
            match inner_seqs.len() {
                0 => {}
                1 => {
                    // there is only one inner sequences, so we can "factor out"
                    // all parameters and use them as inputs to the compound
                    // instruction
                    let params = stackification
                        .getters
                        .remove(&inner_seqs[0].start)
                        .unwrap_or_default();
                    cmpd_inputs.entry(pc).or_default().extend(params);
                }
                2 => {
                    let mut i = 0;
                    while let Some(&a) = stackification
                        .getters
                        .get(&inner_seqs[0].start)
                        .and_then(|v| v.get(i))
                        && let Some(&b) = stackification
                            .getters
                            .get(&inner_seqs[1].start)
                            .and_then(|v| v.get(i))
                        && a == b
                    {
                        // this parameter is common to both inner sequences,
                        // so it can be factored out and used as an input
                        // to the compound instruction
                        stackification
                            .getters
                            .get_mut(&inner_seqs[0].start)
                            .unwrap()
                            .pop_front();
                        stackification
                            .getters
                            .get_mut(&inner_seqs[1].start)
                            .unwrap()
                            .pop_front();
                        cmpd_inputs.entry(pc).or_default().push(a);
                        i += 1;
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
                println!("get ${}", usize::from(*value))
            }
        }

        println!("${} = {:?}", usize::from(pc), insns[pc]);
    }
}

#[cfg(test)]
mod tests {
    use typed_index_collections::ti_vec;

    use super::*;

    use crate::lir::{BinaryOpcode, InsnKind::*, ValType, instructions};

    #[test]
    fn basic_sequence() {
        instructions! {
            let insns;
            a = [constant(I32, 0)];
            b = [constant(I32, 1)];
            c = [add(a, b)];
        }

        let stackification = stackify_lir(&insns);

        assert_eq!(
            stackification.forest,
            ti_vec![
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: c },
                    subtree_start: a,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: c },
                    subtree_start: b,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: a,
                },
            ]
        );
    }

    #[test]
    fn includes_branches() {}
}
