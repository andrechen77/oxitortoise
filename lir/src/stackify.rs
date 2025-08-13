use std::{
    collections::{BTreeMap, VecDeque},
    ops::{Index, IndexMut, Range},
};

mod macros;
pub use macros::stackification;

pub trait InsnUniverse {
    type Pc: Copy + PartialEq + PartialOrd + Ord;

    /// An iterator over all instructions of a certain range, without recursing
    /// into compound instructions in the range.
    fn instructions_in_range(&self, range: Range<Self::Pc>) -> impl Iterator<Item = Self::Pc>;

    /// Returns the instruction that shallowly succeeds the specified
    /// instruction (compound instructions are considered one instruction).
    fn succ_of(&self, insn: Self::Pc) -> Self::Pc;

    /// Which instructions are used by the specified instruction as input,
    /// given in the order they must be on the stack (bottom first).
    fn inputs_of(&self, insn: Self::Pc) -> impl DoubleEndedIterator<Item = Self::Pc>;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OutputMode<Pc> {
    /// This represents an undetermined output type. If this instruction
    /// releases its result onto the operand stack, it will *not* be picked up
    /// before the next available instruction.
    Available,
    /// This instruction will release its result onto the operand stack, and it
    /// will be picked up by the parent instruction before the next available
    /// instruction.
    Release { parent: Pc },
    /// Do not let this instruction release its result onto the operand stack.
    /// The parent instruction is the instruction whose arguments were being
    /// calculated when this instruction was captured.
    Capture { parent: Pc },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Stackification<Pc, M>
where
    M: IndexMut<Pc, Output = InsnTreeInfo<Pc>>,
{
    pub forest: M,
    pub getters: BTreeMap<Pc, VecDeque<Pc>>,
}

/// Information associated with an instruction that helps determine its
/// input/output relations with other instructions.
#[derive(Debug, PartialEq, Eq)]
pub struct InsnTreeInfo<Pc> {
    pub output_mode: OutputMode<Pc>,
    /// The start of the subtree rooted at this instruction.
    pub subtree_start: Pc,
}

/// Finds the root of the subtree that includes `start_insn`. If the next parent
/// encountered would be `ignore_insn`, then stops and returns the previous
/// instruction traversed.
fn find_root<Pc: PartialEq + PartialOrd + Copy, M>(
    forest: &M,
    start_insn: Pc,
    ignore_insn: Pc,
) -> Pc
where
    M: Index<Pc, Output = InsnTreeInfo<Pc>>,
{
    let mut insn = start_insn;
    while let OutputMode::Capture { parent } | OutputMode::Release { parent } =
        forest[insn].output_mode
        && parent != ignore_insn
    {
        insn = parent;
    }
    insn
}

/// Calculates how the given instruction sequence with sequential control flow
/// can be made to execute correctly on a stack machine. This function is only
/// valid for ranges of instructions with sequential control flow, meaning each
/// instruction leads into the next one (or diverges). Assume a stack machine
/// where each instruction, when executed, pops some number of values off the
/// top of the operand stack and pushes the result onto the operand stack. This
/// function calculates the correct points at which to insert operand stack
/// manipulators so that each each LIR instruction in the given sequence, when
/// executed *in that sequence*, will see its inputs on the stack in the correct
/// order. Notice that this function will never re-order instructions: it works
/// with the given order.
///
/// # Returns
///
/// A tuple containing: 0) the output mode for each instruction in the block,
/// and 1) additional places to insert local variable getter. For `return.0`,
/// the output mode of index `i` indicates what manipulators, if any, should be
/// inserted *after* the instruction at index `i` in the block, to get correct
/// stack machine execution. For `return.1`, the tuple `(i, insn)` indicates
/// that a local variable getter should be inserted *before* the instruction at
/// index `i` in the block, to get correct stack machine execution. This list
/// will be sorted by `i`. There may be multiple local variable getters before
/// the same index. In this case they should inserted in the order they appear.
pub fn stackify_sequential<T: InsnUniverse, M: IndexMut<T::Pc, Output = InsnTreeInfo<T::Pc>>>(
    insns: &T,
    seq: Range<T::Pc>,
    stackification: &mut Stackification<T::Pc, M>,
) {
    /*
    Algorithm: All instructions start with an output mode of available. Iterate
    through each LIR instruction, conceptually executing
    them one at a time. When an instruction C needs inputs from previous
    instructions A_0, A_1, A_2, etc. (where they are numbered according to the
    argument order of C), attempt to allow A_0, etc. to release their
    results onto the operand stack in argument order. Let the instruction for
    which we are evaluating release be A_x.

    Releasing A_x cannot be done if one of the following is true.
    - A_x is not available (including if it was already released).
    - A_x was executed before a previous input A_y where y < x. Releasing A_x
    here would cause the operands to be out of order.
    - There exists a previous input A_y where y < x such that the subtree rooted
    at A_x includes A_y. The subtree comprises all instructions which are used
    to build
    up the operand for A_x; on the right it is bordered by and excludes A_x, and
    on the left it is bordered by and includes the subtree for
    A_x's first released argument. Releasing A_x
    would make it impossible to get A_y in the correct place. Since A_y is
    calculated within the subtree, we cannot add a getter for
    A_y before the subtree.

    Based on the above, do the following.
    - If A_x can be released, set its output mode to `Release`
    - If A_x cannot be released, insert a getter manipulator directly after
    A_{x-1}, or directly at the next available sport after the A_x instruction
    itself, whichever comes later; or if A_x is the first argument A_0, directly
    before A_1; or if A_x is the last argument, directly before C. This will
    ensure the argument is in the correct place on the operand stack when C
    executes. Do not change its output mode yet.

    At this point, all arguments A_0, etc. have been accounted for. Now we
    handle all instructions between the released arguments and C.
    Let a be the index of the earliest executed instruction in A_0, etc. that
    was released. Let c be the index of C. For all instructions I in [a, c),
    if I was available, change I's output mode to `Capture`. This will prevent
    any instructions other than those we want to be released from affecting the
    operand stack when C executes.
    */

    for current_insn in insns.instructions_in_range(seq.clone()) {
        #[derive(Clone, Copy)]
        struct SubtreeSpan<Pc> {
            // this is the first input that was released.
            first_released: Pc,
            // this is the pc at which the latest input was released/gotten.
            // inserting a getter or releasing an operand earlier than this point
            // will cause operands to be out of order.
            insertion_pc: Pc,
        }
        let mut subtree_span: Option<SubtreeSpan<T::Pc>> = None;

        // vector of (insertion_pc, value) pairs indicating getters for `value`
        // inserted at `insertion_pc`
        let mut current_getters: Vec<(T::Pc, T::Pc)> = vec![];

        for input_insn in insns.inputs_of(current_insn) {
            // check if the input can be released

            // the input must be inside the current block. assume that
            // instructions only use inputs that are in scope (i.e. does not
            // reach into compound instructions or use inputs from the future),
            // which is required for well-formed instruction sequences.
            if seq.contains(&input_insn)
                // the input must be available
                && stackification.forest[input_insn].output_mode == OutputMode::Available
                // the input must not occur out of order. None compares less
                // than any index
                && Some(input_insn) >= subtree_span.map(|s| s.insertion_pc)
                // the input must be calculated after all previous inputs that
                // need a getter
                && current_getters.iter().all(|&(_, value)| value < stackification.forest[input_insn].subtree_start)
            {
                // the input can be released
                match &mut subtree_span {
                    Some(s) => s.insertion_pc = insns.succ_of(input_insn),
                    None => {
                        subtree_span = Some(SubtreeSpan {
                            first_released: input_insn,
                            insertion_pc: insns.succ_of(input_insn),
                        });

                        // since this is the first input to be released, we need
                        // to move the getters for all previous inputs to be
                        // before this input
                        for (pc, _) in &mut current_getters {
                            *pc = stackification.forest[input_insn].subtree_start;
                        }
                    }
                }
                stackification.forest[input_insn].output_mode = OutputMode::Release {
                    parent: current_insn,
                };
            } else {
                // the input cannot be released. insert a getter for it
                match &mut subtree_span {
                    Some(s) => {
                        // ignore the current instruction when finding the
                        // root because we still haven't finished building
                        // the subtree rooted at the current instruction
                        // yet
                        let root_insn = find_root(&stackification.forest, input_insn, current_insn);
                        if root_insn != current_insn && root_insn > s.insertion_pc {
                            s.insertion_pc = insns.succ_of(root_insn);
                        }
                        current_getters.push((s.insertion_pc, input_insn));
                    }
                    None => {
                        // no inputs have been released yet. we don't know where
                        // to insert the getter, so insert it before the current
                        // instruction for now
                        current_getters.push((current_insn, input_insn));
                    }
                }
            }
        }

        // at this point we have gone through all the inputs of the current
        // instruction. if some inputs were released, then we need to lock up
        // all instructions between the first released input and the current
        // instruction
        if let Some(SubtreeSpan { first_released, .. }) = subtree_span {
            stackification.forest[current_insn].subtree_start =
                stackification.forest[first_released].subtree_start;
            // do not lock the current instruction, since its output type is
            // still undetermined
            for to_be_locked in insns.instructions_in_range(first_released..current_insn) {
                if stackification.forest[to_be_locked].output_mode == OutputMode::Available {
                    stackification.forest[to_be_locked].output_mode = OutputMode::Capture {
                        parent: current_insn,
                    };
                }
            }
        }

        // insert getters required for inputs to the current insn. insert the
        // getters in the order they appear; however they must all be inserted
        // before any other getters at the same index. this is because any
        // getters already inserted are meant to apply to the instruction
        // following them, which binds more tightly than the current
        // instruction.
        for (insertion_pc, value) in current_getters.into_iter().rev() {
            stackification
                .getters
                .entry(insertion_pc)
                .or_default()
                .push_front(value);
        }
    }
    debug_assert!(verify_stackification(insns, seq, &stackification).is_ok());
}

/// Verifies if the given stackfication is correct with respect to the given
/// instruction sequence. Returns an error with the instruction at which an
/// error occurs.
fn verify_stackification<T: InsnUniverse, M: IndexMut<T::Pc, Output = InsnTreeInfo<T::Pc>>>(
    insns: &T,
    seq: Range<T::Pc>,
    stackification: &Stackification<T::Pc, M>,
) -> Result<(), T::Pc> {
    let mut op_stack: Vec<T::Pc> = vec![];

    for insn in insns.instructions_in_range(seq) {
        // first, resolve all local getters before this instruction
        if let Some(values) = stackification.getters.get(&insn) {
            for &value in values {
                // verify that the instruction being gotten has actually been
                // calculcated
                if value >= insn {
                    return Err(insn);
                }

                op_stack.push(value);
            }
        }

        // take input operands off the operand stack
        for input in insns.inputs_of(insn).into_iter().rev() {
            if op_stack.pop() != Some(input) {
                return Err(insn);
            }
        }

        // push the result onto the operand stack unless it is captured
        match stackification.forest[insn].output_mode {
            OutputMode::Release { .. } | OutputMode::Available => op_stack.push(insn),
            OutputMode::Capture { .. } => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct TestInsn {
        inputs: Vec<usize>,
    }
    type TestInsnUniverse = Vec<TestInsn>;
    impl InsnUniverse for TestInsnUniverse {
        type Pc = usize;

        fn instructions_in_range(&self, range: Range<Self::Pc>) -> impl Iterator<Item = Self::Pc> {
            range.start..range.end
        }

        fn inputs_of(&self, insn: Self::Pc) -> impl DoubleEndedIterator<Item = Self::Pc> {
            self[insn].inputs.iter().copied()
        }

        fn succ_of(&self, insn: Self::Pc) -> Self::Pc {
            insn + 1
        }
    }

    fn create_stackification(
        insns: &TestInsnUniverse,
    ) -> Stackification<usize, Vec<InsnTreeInfo<usize>>> {
        Stackification {
            forest: (0..insns.len())
                .map(|i| InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: i,
                })
                .collect(),
            getters: BTreeMap::new(),
        }
    }

    macro_rules! test_insns {
        ($(($idx:expr) [$($input:expr),* $(,)*]),* $(,)*) => {
            vec![
                $(
                    TestInsn {
                        inputs: vec![$($input),* ]
                    },
                )*
            ]
        };
    }

    #[test]
    fn test_insns_macro() {
        let insns = test_insns![
            (0) [1, 2],
            (1) [3, 4, 5],
            (2) [], // Empty inputs
        ];

        assert_eq!(insns.len(), 3);
        assert_eq!(insns[0].inputs, vec![1, 2]);
        assert_eq!(insns[1].inputs, vec![3, 4, 5]);
        assert!(insns[2].inputs.is_empty());
    }

    #[test]
    fn ignore_all() {
        let block = test_insns![
            (0) [],
            (1) [],
            (2) [],
            (3) [],
        ];
        let mut stackification = create_stackification(&block);
        stackify_sequential(&block, 0..block.len(), &mut stackification);
        assert_eq!(
            stackification.forest,
            [
                InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: 0,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: 1,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: 2,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: 3,
                },
            ]
        );
        assert_eq!(stackification.getters, BTreeMap::new());
    }

    #[test]
    fn use_all_in_order() {
        let block = test_insns![
            (0) [],
            (1) [],
            (2) [],
            (3) [],
            (4) [0, 1, 2, 3],
        ];
        let mut stackification = create_stackification(&block);
        stackify_sequential(&block, 0..block.len(), &mut stackification);
        assert_eq!(
            stackification.forest,
            [
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 4 },
                    subtree_start: 0,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 4 },
                    subtree_start: 1,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 4 },
                    subtree_start: 2,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 4 },
                    subtree_start: 3,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: 0,
                },
            ]
        );
        assert_eq!(stackification.getters, BTreeMap::new());
    }

    #[test]
    fn use_skipping_in_order() {
        let block = test_insns![
            (0) [],
            (1) [],
            (2) [],
            (3) [],
            (4) [1, 3],
        ];
        let mut stackification = create_stackification(&block);
        stackify_sequential(&block, 0..block.len(), &mut stackification);
        assert_eq!(
            stackification.forest,
            [
                InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: 0,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 4 },
                    subtree_start: 1,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Capture { parent: 4 },
                    subtree_start: 2,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 4 },
                    subtree_start: 3,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: 1,
                },
            ]
        );
        assert_eq!(stackification.getters, BTreeMap::new());
    }

    #[test]
    fn use_out_of_order() {
        let block = test_insns![
            (0) [],
            (1) [],
            (2) [],
            (3) [],
            (4) [1, 3, 2, 0],
        ];
        let mut stackification = create_stackification(&block);
        stackify_sequential(&block, 0..block.len(), &mut stackification);
        assert_eq!(
            stackification.forest,
            [
                InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: 0,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 4 },
                    subtree_start: 1,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Capture { parent: 4 },
                    subtree_start: 2,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 4 },
                    subtree_start: 3,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: 1,
                },
            ]
        );
        assert_eq!(
            stackification.getters,
            BTreeMap::from([(4, VecDeque::from([2, 0]))]),
        );
    }

    #[test]
    fn use_repeated() {
        let block = test_insns![
            (0) [],
            (1) [],
            (2) [0, 0, 1],
        ];
        let mut stackification = create_stackification(&block);
        stackify_sequential(&block, 0..block.len(), &mut stackification);
        assert_eq!(
            stackification.forest,
            [
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 2 },
                    subtree_start: 0,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 2 },
                    subtree_start: 1,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: 0,
                },
            ]
        );
        assert_eq!(
            stackification.getters,
            BTreeMap::from([(1, VecDeque::from([0]))])
        );
    }

    #[test]
    fn queueing_getters() {
        let block = test_insns![
            (0) [],
            (1) [],
            (2) [0, 1],
            (3) [],
            (4) [1, 0, 3],
        ];
        let mut stackification = create_stackification(&block);
        stackify_sequential(&block, 0..block.len(), &mut stackification);
        assert_eq!(
            stackification.forest,
            [
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 2 },
                    subtree_start: 0,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 2 },
                    subtree_start: 1,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: 0,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 4 },
                    subtree_start: 3,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: 3,
                },
            ]
        );
        assert_eq!(
            stackification.getters,
            BTreeMap::from([(3, VecDeque::from([1, 0]))])
        );
    }

    #[test]
    fn staircase() {
        let block = test_insns![
            (0) [],
            (1) [0],
            (2) [0, 1],
            (3) [0, 1, 1, 2],
        ];
        let mut stackification = create_stackification(&block);
        stackify_sequential(&block, 0..block.len(), &mut stackification);
        assert_eq!(
            stackification.forest,
            [
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 1 },
                    subtree_start: 0,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: 0,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 3 },
                    subtree_start: 2,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: 2,
                },
            ]
        );
        assert_eq!(
            stackification.getters,
            BTreeMap::from([(2, VecDeque::from([0, 1, 1, 0, 1]))])
        );
    }

    #[test]
    fn nested_operators() {
        let block = test_insns![
            (0) [],
            (1) [],
            (2) [0, 1],
            (3) [1],
            (4) [],
            (5) [4],
            (6) [2, 5],
            (7) [0, 6],
        ];
        let mut stackification = create_stackification(&block);
        stackify_sequential(&block, 0..block.len(), &mut stackification);
        assert_eq!(
            stackification.forest,
            [
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 2 },
                    subtree_start: 0,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 2 },
                    subtree_start: 1,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 6 },
                    subtree_start: 0,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Capture { parent: 6 },
                    subtree_start: 3,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 5 },
                    subtree_start: 4,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 6 },
                    subtree_start: 4,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: 0,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: 7,
                },
            ]
        );
        assert_eq!(
            stackification.getters,
            BTreeMap::from([(3, VecDeque::from([1])), (7, VecDeque::from([0, 6]))])
        );
    }

    #[test]
    fn external_instructions() {
        let block = test_insns![
            (0) [],
            (1) [],
            (2) [],
            (3) [],
            (4) [],
            (5) [],
            (6) [],
            (7) [],
            (8) [],
            (9) [],
            (10) [9],
            (11) [10],
            (12) [11, 10],
        ];
        let mut stackification = create_stackification(&block);
        stackify_sequential(&block, 10..block.len(), &mut stackification);
        assert_eq!(
            stackification.forest[10..],
            [
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 11 },
                    subtree_start: 10,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 12 },
                    subtree_start: 10,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: 10,
                },
            ]
        );
        assert_eq!(
            stackification.getters,
            BTreeMap::from([(10, VecDeque::from([9])), (12, VecDeque::from([10]))])
        );
    }

    #[test]
    fn conflicting_getter_locations() {
        let block = test_insns![
            (0) [],
            (1) [],
            (2) [],
            (3) [],
            (4) [],
            (5) [],
            (6) [],
            (7) [],
            (8) [],
            (9) [],
            (10) [],
            (11) [],
            (12) [10, 11],
            (13) [10],
            (14) [12, 11, 13],
        ];
        let mut stackification = create_stackification(&block);
        stackify_sequential(&block, 10..block.len(), &mut stackification);
        assert_eq!(
            stackification.forest[10..],
            [
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 12 },
                    subtree_start: 10,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 12 },
                    subtree_start: 11,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 14 },
                    subtree_start: 10,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 14 },
                    subtree_start: 13,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: 10,
                },
            ]
        );
        assert_eq!(
            stackification.getters,
            BTreeMap::from([(13, VecDeque::from([11, 10]))])
        );
    }

    #[test]
    fn getter_before_calculation_after_release() {
        let block = test_insns![
            (0) [],
            (1) [],
            (2) [],
            (3) [1, 2],
            (4) [3],
            (5) [0, 3],
        ];

        let mut stackification = create_stackification(&block);
        stackify_sequential(&block, 0..block.len(), &mut stackification);
        assert_eq!(
            stackification.forest,
            [
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 5 },
                    subtree_start: 0,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 3 },
                    subtree_start: 1,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 3 },
                    subtree_start: 2,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 4 },
                    subtree_start: 1,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Capture { parent: 5 },
                    subtree_start: 1,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: 0,
                },
            ]
        );
        assert_eq!(
            stackification.getters,
            BTreeMap::from([(5, VecDeque::from([3]))])
        );
    }

    #[test]
    fn getter_before_calculation_before_release() {
        let block = test_insns![
            (0) [],
            (1) [],
            (2) [],
            (3) [1, 2],
            (4) [3],
            (5) [3, 0],
        ];

        let mut stackification = create_stackification(&block);
        stackify_sequential(&block, 0..block.len(), &mut stackification);
        assert_eq!(
            stackification.forest,
            [
                InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: 0,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 3 },
                    subtree_start: 1,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 3 },
                    subtree_start: 2,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Release { parent: 4 },
                    subtree_start: 1,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: 1,
                },
                InsnTreeInfo {
                    output_mode: OutputMode::Available,
                    subtree_start: 5,
                },
            ]
        );
        assert_eq!(
            stackification.getters,
            BTreeMap::from([(5, VecDeque::from([3, 0]))])
        );
    }
}
