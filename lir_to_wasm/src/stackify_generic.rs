use std::ops::Add;
use std::{collections::HashMap, hash::Hash};

mod macros;
use lir::smallvec::SmallVec;
use lir::typed_index_collections::TiVec;
#[allow(unused_imports)]
pub use macros::stackification;

/*
stackification model

the function is given an iterator over (idx, input, output). each one of these
means that at the given idx, an instruction is executed that expects the given
inputs on the operand stack and pushes the given outputs onto the operand stack.
A regular instruction will just push one output, representing the value of the
instruction's calculation, onto the stack, but other instructions might push
multiple values, or propagate values from other instructions. A value is
immutable, so any reference to a value can be satisfied by any instance of that
value: e.g. if instruction A pushes a value A.v onto the stack, and then an
identity instruction B pops A.v and pushes A.v again, then future instructions
can use the output of B if it expects A.v.

The goal of the function is to return a list of values, associated with each idx,
indicating the number of captures at that idx (indicating operands removed from
the stack) and getters to insert at that idx, before the instruction at that idx
runs. It also returns a list of values representing the available operands on
the stack at the end of the sequence,
*/

#[derive(Debug, PartialEq, Eq)]
pub struct InsnSeqStackification<V, Idx: From<usize>> {
    /// Values that must be on the stack when entering this instruction
    /// sequence to get proper stack machine execution.
    pub inputs: Vec<V>,
    /// Information about execution at the specified idx to get proper stack
    /// machine execution.
    pub manips: TiVec<Idx, StackManipulators<V>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AvailOperand<V, Idx> {
    value: V,
    /// The index at which the operand can be released (one plus the index of the
    /// instruction that outputs it, or zero if the operand is a forced
    /// argument). These values are monotonically increasing over the
    /// multivalues on the operand stack.
    releases_at: Idx,
    /// The index at which the direct calculation of this operand, represented as
    /// a tree, begins. Retro-inserting something onto the stack before this
    /// operand can be done no later than this index.
    subtree_start: Idx,
    /// If true, then this operand ends a multivalue. Each single value operand
    /// ends a multivalue with just itself. All values of a multivalue except
    /// for the last one will have this set to false. A getter can only be
    /// inserted before an operand that ends a multivalue. If value in a
    /// multivalue is captured, then all following values in the same multivalue
    /// must also be captured. A multivalue on the stack (a sequence of [false,
    /// false, ..., true]) does not necessarily have to come from a single
    /// instruction. For example, if an instruction consumes only the suffix of
    /// a multivalue and then outputs its own value on the stack, then it will
    /// create a Frankenstein multivalue consisting of the unconsumed prefix of
    /// the original multivalue concatenated with the return value of the
    /// just-executed instruction. This is okay and is treated the same as any
    /// other multivalue.
    ends_multivalue: bool,
}

/// Describes what stack manipulators exist/need to be added at a given idx.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StackManipulators<V> {
    /// The number of captures to insert at this idx. A "capture" corresponds to
    /// popping a value off the top of the stack and dropping it. These captures
    /// are inserted first before any other stack manipulators and before the
    /// instruction at the idx is executed.
    pub captures: usize,
    /// The values for the getters to insert at this idx. These are inserted
    /// after the captures but before the instruction at the idx is executed.
    pub getters: Vec<V>,
    /// The inputs to the instruction at this idx.
    pub inputs: SmallVec<[V; 2]>,
    /// The outputs of the instruction at this idx.
    pub outputs: SmallVec<[V; 1]>,
}

// impl<V> Default for StackManipulators<V> {
//     fn default() -> Self {
//         Self { captures: 0, getters: Vec::new() }
//     }
// }

/// * `available_operand_stack` - The stack of operands that are available to
///   the current instruction.
/// * `available_getters` - Maps each value to the earliest idx at which it can
///   be gotten. None indicates that the value was calculated before any idx.
/// * `my_idx` - The idx at which the instruction is executed. Must be greater
///   than all idx's in the available_operands stack and available_getters
pub fn stackify_single<V, Idx>(
    available_operand_stack: &mut Vec<AvailOperand<V, Idx>>,
    available_getters: &mut HashMap<V, Idx>,
    stack_manips: &mut TiVec<Idx, StackManipulators<V>>,
    my_idx: Idx,
    inputs: SmallVec<[V; 2]>,
    outputs: SmallVec<[V; 1]>,
) where
    V: Copy + PartialEq + Eq + Hash,
    Idx: PartialOrd + Ord + Copy + From<usize> + Add<usize, Output = Idx>,
    usize: From<Idx>,
{
    // The order barrier is an index into the operand stack. Only operands
    // after the order barrier may be released. Getters may only be inserted
    // after the order barrier. This restriction ensures that the inputs to
    // the current instruction are in the correct order.
    let mut order_barrier = 0;
    // An index to the operand stack indicating the first operand to be
    // released.
    let mut first_released_operand = None;

    let mut operand_is_released = vec![false; available_operand_stack.len()];

    // If we haven't released any inputs yet, then we won't know where to insert
    // a getter even if know we need a getter. There are called "lost getters",
    // and will be stored at the idx of the current instruction if there are no
    // released inputs, and before the first released input if there are.
    let mut getters = Vec::new();

    for input in &inputs {
        // check if the input is available on the operand stack after the
        // order barrier
        if let Some((operand_idx, operand)) =
            available_operand_stack.iter().enumerate().skip(order_barrier).find(|(_, o)| o.value == *input)
            // check if the subtree start of the input comes after all previous
            // inputs with lost getters
            && getters.iter().all(|(_, v)| available_getters.get(v) <= Some(&operand.subtree_start))
        {
            // the input can be released
            operand_is_released[operand_idx] = true;

            if first_released_operand.is_none() {
                first_released_operand = Some(operand_idx);

                // since this is the first input to be released, we need to move
                // the getters for all previous inputs to be before this input.
                // the above check that the subtree start comes after all
                // previous getter-needing inputs ensures that the getters are
                // valid at their new position.
                for (idx, _) in &mut getters {
                    *idx = operand.subtree_start;
                }
            }

            // update the order barrier on the operand stack to right after this
            // input. this ensures that the next inputs to the instruction are
            // in the correct order
            order_barrier = operand_idx + 1;
        } else {
            // the input cannot be released. insert a getter for it.

            // get the idx after which the value can be gotten.
            let birthdate = available_getters.get(input);
            assert!(birthdate <= Some(&my_idx));

            if first_released_operand.is_some() {
                // a previous input has already been released. try to insert the
                // getter as early as possible after the most recently released
                // input while respecting the order of inputs

                let (new_order_barrier, insertion_idx) = (order_barrier..)
                    .map(|i| (i, &available_operand_stack[i - 1])) // order barrier must be > 0 bc something was released
                    .find_map(|(i, o)| {
                        (o.ends_multivalue && Some(&o.releases_at) >= birthdate)
                            .then_some((i, o.releases_at))
                    })
                    // if the top of the stack is just part of a multivalue,
                    // the suffix of which was chomped off by someone else
                    // before we got here, then we can't insert a getter after
                    // that multivalue was released. therefore, put the getter
                    // before this instruction.
                    .unwrap_or((available_operand_stack.len(), my_idx));

                // update the order barrier to ensure that the next input is in
                // the correct order
                order_barrier = new_order_barrier;

                getters.push((insertion_idx, *input));
            } else {
                // no previous inputs have been released yet. we don't know
                // where to insert the getter, so store it before the current
                // instruction for now
                getters.push((my_idx, *input));
            };
        }
    }

    let my_subtree_start =
        first_released_operand.map_or(my_idx, |i| available_operand_stack[i].subtree_start);

    // at this point we have gone through all the inputs of the current
    // instruction.

    // if some inputs were released, then we need to remove all operands on the
    // stack after the first released operand, capturing where appropriate.
    if let Some(first_released_operand) = first_released_operand {
        remove_excess_operands(
            available_operand_stack
                .drain(first_released_operand..)
                .zip(operand_is_released.drain(first_released_operand..)),
            stack_manips,
            my_idx,
        );
    }
    // in addition, insert all getters at the correct locations. the getters are
    // inserted in the order they appear, but they must come before any getters
    // already inserted at the same idx. The pre-existing getters should bind
    // more tightly, which is done by having them come after
    for (insertion_idx, v) in getters.into_iter().rev() {
        // FIXME can be made faster
        stack_manips[insertion_idx].getters.insert(0, v);
    }

    // at this point we have removed all inputs for the current instruction.
    // now we need to add the outputs of the current instruction to the
    // operand stack for the next instruction to potentially use
    let succ_idx = my_idx + 1;
    for (i, output) in outputs.iter().enumerate() {
        available_operand_stack.push(AvailOperand {
            value: *output,
            releases_at: succ_idx,
            subtree_start: my_subtree_start,
            ends_multivalue: i == outputs.len() - 1,
        });
        available_getters.insert(*output, succ_idx);
    }

    stack_manips[my_idx].inputs = inputs;
    stack_manips[my_idx].outputs = outputs;
}

/// Given that a subsequence of instructions has been stackified and results in
/// the given stack of excess operands, add manipulators such that there is no
/// excess except for the operands marked as released.
///
/// # Arguments
///
/// * `excess_op_stack` - An iterator over all operands needing to be cleared,
///   paired with a boolean indicating whether the operand should be released.
///   False means the operand is excess and should be captured, while true means
///   the operand is required and should be released.
/// * `manips` - The set of stack manipulators to add to such to achieve the
///   desired result.
/// * `target_idx` - The idx at which we want to achieve the target state with the
///   operand stack containing only the released operands. This is also the
///   fallback location to add stack manipulators if manipulators cannot be added
///   immediately after the instructions that produced the operands. This value
///   must be greater than all idx's in the excess op stack.
pub fn remove_excess_operands<V, Idx>(
    excess_op_stack: impl Iterator<Item = (AvailOperand<V, Idx>, bool)>,
    manips: &mut TiVec<Idx, StackManipulators<V>>,
    target_idx: Idx,
) where
    V: Copy + PartialEq + Eq + Hash,
    Idx: PartialOrd + Ord + Copy + From<usize>,
    usize: From<Idx>,
{
    // The general idea of the algorithm is to go through each operand and, if
    // it is required, release it, and if it is excess, capture it. This can
    // usually be done by simply adding or not adding a capture at the idx where
    // the operand would be released.
    //
    // However, if the operand being captured is not the last operand in a
    // multivalue, then all following operands in the same multivalue must also
    // be captured. if one such operand was supposed to be released, then it
    // will instead be captured and then re-released at the end of the
    // multivalue. this process is known as "re-releasing" multivalues. The
    // variable `queued_for_rerelease` is None if a rerelease is not in
    // progress, otherwise it is Some.
    let mut queued_for_rerelease: Option<Vec<V>> = None;
    let mut num_captures = 0;

    for (operand, should_be_released) in excess_op_stack {
        if should_be_released {
            if let Some(queued) = &mut queued_for_rerelease {
                // insert a capture to account for the operand to prevent it
                // from being released, but queue it for re-release so that
                // it will show up as normal when the multivalue ends
                num_captures += 1;
                queued.push(operand.value);
            } else {
                // release normally, which is done by simply not adding
                // a capture
            }
        } else {
            // insert a capture for the operand to prevent it from being
            // released
            num_captures += 1;

            if !operand.ends_multivalue {
                queued_for_rerelease.get_or_insert_default();
            }
        }

        if operand.ends_multivalue {
            // now re-release all the queued operands and add the captures
            // we counted
            if queued_for_rerelease.as_ref().is_some_and(|v| !v.is_empty()) || num_captures > 0 {
                let manips = &mut manips[operand.releases_at];
                if let Some(v) = queued_for_rerelease.take() {
                    manips.getters.extend(v.into_iter());
                }
                manips.captures += num_captures;
            }

            queued_for_rerelease = None;
            num_captures = 0;
        }
    }

    if let Some(queued) = queued_for_rerelease {
        // this means the last operand did not end a multivalue. whatever
        // captures and queued re-releases we counted cannot be inserted
        // at the end of the multivalue because the end was used by a
        // previous instruction. instead, insert the captures and getters
        // immediately before the target idx
        let manips = &mut manips[target_idx];
        manips.getters.extend(queued);
        manips.captures += num_captures;
    }
}

#[cfg(test)]
mod tests {
    use lir::smallvec::{ToSmallVec as _, smallvec};
    use lir::typed_index_collections::ti_vec;

    use super::*;

    type Idx = usize;
    type V = &'static str;

    macro_rules! avail_operand {
        ($value:expr, $range:expr, $ends_multivalue:expr) => {
            AvailOperand {
                value: $value,
                releases_at: $range.end,
                subtree_start: $range.start,
                ends_multivalue: $ends_multivalue,
            }
        };
    }

    #[test]
    fn no_op() {
        let mut op_stack = vec![avail_operand!("a", 0..0, true), avail_operand!("b", 0..1, true)];
        let mut getters = HashMap::from([("a", 0), ("b", 1)]);
        let mut stk = stackification! {
            inputs [];
            [0] cap(0) get[] [] => [];
            [1] cap(0) get[] [] => [];
            [2] cap(0) get[] [] => [];
        };

        stackify_single::<V, Idx>(
            &mut op_stack,
            &mut getters,
            &mut stk.manips,
            2,
            smallvec![],
            smallvec![],
        );
        assert_eq!(
            op_stack,
            vec![avail_operand!("a", 0..0, true), avail_operand!("b", 0..1, true)]
        );
        assert_eq!(getters, HashMap::from([("a", 0), ("b", 1)]));
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => [];
                [1] cap(0) get[] [] => [];
                [2] cap(0) get[] [] => [];
            }
        );
    }

    #[test]
    fn three_args_one_output() {
        let mut op_stack = vec![
            avail_operand!("a", 0..0, true),
            avail_operand!("b", 0..1, true),
            avail_operand!("c", 1..2, true),
        ];
        let mut getters = HashMap::from([("a", 0), ("b", 1), ("c", 2)]);
        let mut stk = stackification! {
            inputs [];
            [0] cap(0) get[] [] => [];
            [1] cap(0) get[] [] => [];
            [2] cap(0) get[] [] => [];
        };

        stackify_single::<V, Idx>(
            &mut op_stack,
            &mut getters,
            &mut stk.manips,
            2,
            smallvec!["a", "b", "c"],
            smallvec!["x"],
        );

        assert_eq!(op_stack, vec![avail_operand!("x", 0..3, true)]);
        assert_eq!(getters, HashMap::from([("a", 0), ("b", 1), ("c", 2), ("x", 3)]));
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => [];
                [1] cap(0) get[] [] => [];
                [2] cap(0) get[] ["a", "b", "c"] => ["x"];
            }
        );
    }

    #[test]
    fn args_out_of_order() {
        let mut op_stack = vec![
            avail_operand!("a", 0..0, true),
            avail_operand!("b", 0..1, true),
            avail_operand!("c", 1..2, true),
            avail_operand!("d", 2..3, true),
        ];
        let mut getters = HashMap::from([("a", 0), ("b", 1), ("c", 2), ("d", 3)]);
        let mut stk = stackification! {
            inputs [];
            [0] cap(0) get[] [] => [];
            [1] cap(0) get[] [] => [];
            [2] cap(0) get[] [] => [];
            [3] cap(0) get[] [] => [];
        };

        stackify_single::<V, Idx>(
            &mut op_stack,
            &mut getters,
            &mut stk.manips,
            3,
            smallvec!["c", "b", "d"],
            smallvec!["x"],
        );

        assert_eq!(
            op_stack,
            vec![
                avail_operand!("a", 0..0, true),
                avail_operand!("b", 0..1, true),
                avail_operand!("x", 1..4, true),
            ]
        );
        assert_eq!(getters, HashMap::from([("a", 0), ("b", 1), ("c", 2), ("d", 3), ("x", 4)]));
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => [];
                [1] cap(0) get[] [] => [];
                [2] cap(0) get["b"] [] => [];
                [3] cap(0) get[] ["c", "b", "d"] => ["x"];
            }
        );
    }

    #[test]
    fn stackify_args_with_captures() {
        let mut op_stack = vec![
            avail_operand!("a", 0..0, true),
            avail_operand!("b", 0..1, true),
            avail_operand!("c", 1..2, true),
            avail_operand!("d", 2..3, true),
        ];
        let mut getters = HashMap::from([("a", 0), ("b", 1), ("c", 2), ("d", 3)]);
        let mut stk = stackification! {
            inputs [];
            [0] cap(0) get[] [] => [];
            [1] cap(0) get[] [] => [];
            [2] cap(0) get[] [] => [];
            [3] cap(0) get[] [] => [];
        };

        stackify_single::<V, Idx>(
            &mut op_stack,
            &mut getters,
            &mut stk.manips,
            3,
            smallvec!["b", "d"],
            smallvec!["x"],
        );

        assert_eq!(
            op_stack,
            vec![avail_operand!("a", 0..0, true), avail_operand!("x", 0..4, true),]
        );
        assert_eq!(getters, HashMap::from([("a", 0), ("b", 1), ("c", 2), ("d", 3), ("x", 4)]));
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => [];
                [1] cap(0) get[] [] => [];
                [2] cap(1) get[] [] => [];
                [3] cap(0) get[] ["b", "d"] => ["x"];
            }
        );
    }

    #[test]
    fn multivalue_output() {
        let mut op_stack = vec![
            avail_operand!("a", 0..0, true),
            avail_operand!("b", 0..1, true),
            avail_operand!("c", 1..2, true),
            avail_operand!("d", 2..3, true),
        ];
        let mut getters = HashMap::from([("a", 0), ("b", 1), ("c", 2), ("d", 3)]);
        let mut stk = stackification! {
            inputs [];
            [0] cap(0) get[] [] => [];
            [1] cap(0) get[] [] => [];
            [2] cap(0) get[] [] => [];
            [3] cap(0) get[] [] => [];
        };

        stackify_single::<V, Idx>(
            &mut op_stack,
            &mut getters,
            &mut stk.manips,
            3,
            smallvec!["c"],
            smallvec!["x", "y", "z"],
        );

        assert_eq!(
            op_stack,
            vec![
                avail_operand!("a", 0..0, true),
                avail_operand!("b", 0..1, true),
                avail_operand!("x", 1..4, false),
                avail_operand!("y", 1..4, false),
                avail_operand!("z", 1..4, true),
            ]
        );
        assert_eq!(
            getters,
            HashMap::from([("a", 0), ("b", 1), ("c", 2), ("d", 3), ("x", 4), ("y", 4), ("z", 4)])
        );
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => [];
                [1] cap(0) get[] [] => [];
                [2] cap(0) get[] [] => [];
                [3] cap(1) get[] ["c"] => ["x", "y", "z"];
            }
        );
    }

    #[test]
    fn multivalue_input_whole() {
        let mut op_stack = vec![
            avail_operand!("a", 0..0, true),
            avail_operand!("b", 0..1, true),
            avail_operand!("c", 1..2, false),
            avail_operand!("d", 1..2, false),
            avail_operand!("e", 1..2, true),
            avail_operand!("f", 2..3, true),
        ];
        let mut getters =
            HashMap::from([("a", 0), ("b", 1), ("c", 2), ("d", 2), ("e", 2), ("f", 3)]);
        let mut stk = stackification! {
            inputs [];
            [0] cap(0) get[] [] => [];
            [1] cap(0) get[] [] => [];
            [2] cap(0) get[] [] => [];
            [3] cap(0) get[] [] => [];
        };

        stackify_single::<V, Idx>(
            &mut op_stack,
            &mut getters,
            &mut stk.manips,
            3,
            smallvec!["c", "d", "e", "f"],
            smallvec!["x"],
        );

        assert_eq!(
            op_stack,
            vec![
                avail_operand!("a", 0..0, true),
                avail_operand!("b", 0..1, true),
                avail_operand!("x", 1..4, true)
            ]
        );
        assert_eq!(
            getters,
            HashMap::from([("a", 0), ("b", 1), ("c", 2), ("d", 2), ("e", 2), ("f", 3), ("x", 4)])
        );
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => [];
                [1] cap(0) get[] [] => [];
                [2] cap(0) get[] [] => [];
                [3] cap(0) get[] ["c", "d", "e", "f"] => ["x"];
            }
        );
    }

    #[test]
    fn multivalue_input_prefix() {
        let mut op_stack = vec![
            avail_operand!("a", 0..0, true),
            avail_operand!("b", 0..1, false),
            avail_operand!("c", 0..1, false),
            avail_operand!("d", 0..1, true),
        ];
        let mut getters = HashMap::from([("a", 0), ("b", 1), ("c", 1), ("d", 1)]);
        let mut stk = stackification! {
            inputs [];
            [0] cap(0) get[] [] => [];
            [1] cap(0) get[] [] => [];
        };

        stackify_single::<V, Idx>(
            &mut op_stack,
            &mut getters,
            &mut stk.manips,
            1,
            smallvec!["b", "c"],
            smallvec!["x"],
        );

        assert_eq!(
            op_stack,
            vec![avail_operand!("a", 0..0, true), avail_operand!("x", 0..2, true)]
        );
        assert_eq!(getters, HashMap::from([("a", 0), ("b", 1), ("c", 1), ("d", 1), ("x", 2)]));
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => [];
                [1] cap(1) get[] ["b", "c"] => ["x"];
            }
        );
    }

    #[test]
    fn multivalue_input_suffix() {
        let mut op_stack = vec![
            avail_operand!("a", 0..0, true),
            avail_operand!("b", 0..1, false),
            avail_operand!("c", 0..1, false),
            avail_operand!("d", 0..1, true),
            avail_operand!("e", 1..2, true),
        ];
        let mut getters = HashMap::from([("a", 0), ("b", 1), ("c", 1), ("d", 1), ("e", 2)]);
        let mut stk = stackification! {
            inputs [];
            [0] cap(0) get[] [] => [];
            [1] cap(0) get[] [] => [];
            [2] cap(0) get[] [] => [];
        };

        stackify_single::<V, Idx>(
            &mut op_stack,
            &mut getters,
            &mut stk.manips,
            2,
            smallvec!["c", "d", "e"],
            smallvec!["x"],
        );

        assert_eq!(
            op_stack,
            vec![
                avail_operand!("a", 0..0, true),
                avail_operand!("b", 0..1, false),
                avail_operand!("x", 0..3, true)
            ]
        );
        assert_eq!(
            getters,
            HashMap::from([("a", 0), ("b", 1), ("c", 1), ("d", 1), ("e", 2), ("x", 3)])
        );
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => [];
                [1] cap(0) get[] [] => [];
                [2] cap(0) get[] ["c", "d", "e"] => ["x"];
            }
        );
    }

    #[test]
    fn multivalue_input_interleaved() {
        let mut op_stack = vec![
            avail_operand!("a", 0..0, true),
            avail_operand!("b", 0..1, true),
            avail_operand!("c", 1..2, true),
            avail_operand!("d", 2..3, false),
            avail_operand!("e", 2..3, false),
            avail_operand!("f", 2..3, false),
            avail_operand!("g", 2..3, true),
            avail_operand!("h", 3..4, true),
            avail_operand!("i", 4..5, true),
        ];
        let mut getters = HashMap::from([
            ("a", 0),
            ("b", 1),
            ("c", 2),
            ("d", 3),
            ("e", 3),
            ("f", 3),
            ("g", 3),
            ("h", 4),
            ("i", 5),
        ]);
        let mut stk = stackification! {
            inputs [];
            [0] cap(0) get[] [] => [];
            [1] cap(0) get[] [] => [];
            [2] cap(0) get[] [] => [];
            [3] cap(0) get[] [] => [];
            [4] cap(0) get[] [] => [];
            [5] cap(0) get[] [] => [];
        };

        stackify_single::<V, Idx>(
            &mut op_stack,
            &mut getters,
            &mut stk.manips,
            5,
            smallvec!["b", "e", "g", "i"],
            smallvec!["x"],
        );

        assert_eq!(
            op_stack,
            vec![avail_operand!("a", 0..0, true), avail_operand!("x", 0..6, true),]
        );
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => [];
                [1] cap(0) get[] [] => [];
                [2] cap(1) get[] [] => [];
                [3] cap(4) get["e", "g"] [] => [];
                [4] cap(1) get[] [] => [];
                [5] cap(0) get[] ["b", "e", "g", "i"] => ["x"];
            }
        );
    }

    #[test]
    fn multivalue_input_partial_interleaved() {
        let mut op_stack = vec![
            avail_operand!("a", 0..0, true),
            avail_operand!("b", 0..1, true),
            avail_operand!("c", 1..2, true),
            avail_operand!("d", 2..3, false),
            avail_operand!("e", 2..3, false),
            avail_operand!("f", 2..3, false),
            avail_operand!("g", 2..3, true),
            avail_operand!("h", 3..4, true),
            avail_operand!("i", 4..5, true),
        ];
        let mut getters = HashMap::from([
            ("a", 0),
            ("b", 1),
            ("c", 2),
            ("d", 3),
            ("e", 3),
            ("f", 3),
            ("g", 3),
            ("h", 4),
            ("i", 5),
        ]);
        let mut stk = stackification! {
            inputs [];
            [0] cap(0) get[] [] => [];
            [1] cap(0) get[] [] => [];
            [2] cap(0) get[] [] => [];
            [3] cap(0) get[] [] => [];
            [4] cap(0) get[] [] => [];
            [5] cap(0) get[] [] => [];
        };

        stackify_single::<V, Idx>(
            &mut op_stack,
            &mut getters,
            &mut stk.manips,
            5,
            smallvec!["e", "g", "i"],
            smallvec!["x"],
        );

        assert_eq!(
            op_stack,
            vec![
                avail_operand!("a", 0..0, true),
                avail_operand!("b", 0..1, true),
                avail_operand!("c", 1..2, true),
                avail_operand!("d", 2..3, false),
                avail_operand!("x", 2..6, true),
            ]
        );
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => [];
                [1] cap(0) get[] [] => [];
                [2] cap(0) get[] [] => [];
                [3] cap(2) get["g"] [] => [];
                [4] cap(1) get[] [] => [];
                [5] cap(0) get[] ["e", "g", "i"] => ["x"];
            }
        );
    }

    #[test]
    fn multivalue_frankenstein_input() {
        let mut op_stack = vec![
            avail_operand!("a", 0..0, true),
            avail_operand!("b", 0..1, true),
            avail_operand!("c", 1..2, true),
            avail_operand!("d", 2..3, false),
            avail_operand!("x", 2..6, false),
            avail_operand!("y", 2..6, false),
            avail_operand!("z", 2..6, true),
        ];
        let mut getters =
            HashMap::from([("a", 0), ("b", 1), ("c", 2), ("d", 3), ("x", 4), ("y", 4), ("z", 5)]);
        let mut stk = stackification! {
            inputs [];
            [0] cap(0) get[] [] => [];
            [1] cap(0) get[] [] => [];
            [2] cap(0) get[] [] => [];
            [3] cap(0) get[] [] => [];
            [4] cap(0) get[] [] => [];
            [5] cap(0) get[] [] => [];
            [6] cap(0) get[] [] => [];
        };

        stackify_single::<V, Idx>(
            &mut op_stack,
            &mut getters,
            &mut stk.manips,
            6,
            smallvec!["y", "d"],
            smallvec!["w"],
        );

        assert_eq!(
            op_stack,
            vec![
                avail_operand!("a", 0..0, true),
                avail_operand!("b", 0..1, true),
                avail_operand!("c", 1..2, true),
                avail_operand!("d", 2..3, false),
                avail_operand!("x", 2..6, false),
                avail_operand!("w", 2..7, true),
            ]
        );
        assert_eq!(
            getters,
            HashMap::from([
                ("a", 0),
                ("b", 1),
                ("c", 2),
                ("d", 3),
                ("x", 4),
                ("y", 4),
                ("z", 5),
                ("w", 7),
            ])
        );
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => [];
                [1] cap(0) get[] [] => [];
                [2] cap(0) get[] [] => [];
                [3] cap(0) get[] [] => [];
                [4] cap(0) get[] [] => [];
                [5] cap(0) get[] [] => [];
                [6] cap(1) get["d"] ["y", "d"] => ["w"];
            }
        );
    }

    #[test]
    fn repeated_inputs() {
        let mut op_stack = vec![
            avail_operand!("a", 0..0, true),
            avail_operand!("b", 0..1, true),
            avail_operand!("c", 1..2, true),
        ];
        let mut getters = HashMap::from([("a", 0), ("b", 1), ("c", 2)]);
        let mut stk = stackification! {
            inputs [];
            [0] cap(0) get[] [] => [];
            [1] cap(0) get[] [] => [];
            [2] cap(0) get[] [] => [];
        };

        stackify_single::<V, Idx>(
            &mut op_stack,
            &mut getters,
            &mut stk.manips,
            2,
            smallvec!["b", "b"],
            smallvec!["x"],
        );

        assert_eq!(
            op_stack,
            vec![avail_operand!("a", 0..0, true), avail_operand!("x", 0..3, true)]
        );
        assert_eq!(getters, HashMap::from([("a", 0), ("b", 1), ("c", 2), ("x", 3)]));
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => [];
                [1] cap(0) get["b"] [] => [];
                [2] cap(1) get[] ["b", "b"] => ["x"];
            }
        );
    }

    #[test]
    fn repeated_inputs_with_in_between() {
        let mut op_stack = vec![
            avail_operand!("a", 0..0, true),
            avail_operand!("b", 0..1, true),
            avail_operand!("c", 1..2, true),
        ];
        let mut getters = HashMap::from([("a", 0), ("b", 1), ("c", 2)]);
        let mut stk = stackification! {
            inputs [];
            [0] cap(0) get[] [] => [];
            [1] cap(0) get[] [] => [];
            [2] cap(0) get[] [] => [];
        };

        stackify_single::<V, Idx>(
            &mut op_stack,
            &mut getters,
            &mut stk.manips,
            2,
            smallvec!["b", "a", "b"],
            smallvec!["x"],
        );
        assert_eq!(
            op_stack,
            vec![avail_operand!("a", 0..0, true), avail_operand!("x", 0..3, true)]
        );
        assert_eq!(getters, HashMap::from([("a", 0), ("b", 1), ("c", 2), ("x", 3)]));
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => [];
                [1] cap(0) get["a", "b"] [] => [];
                [2] cap(1) get[] ["b", "a", "b"] => ["x"];
            }
        );
    }

    macro_rules! test_insns {
        ($([$($input:expr),*] => [$($output:expr),*]),* $(,)?) => {
            [
                $(
                    (&[$($input),*], &[$($output),*]),
                )*
            ]
        };
    }

    // FIXME change the function to not need external values, since the algorithm
    // above was changed to interpret any value without an entry in
    // `available_getters` as being always available
    fn test_skeleton(external: &[V], insns: &[(&[V], &[V])]) -> InsnSeqStackification<V, Idx> {
        // each sequence starts with fresh operand stack and manipulators
        let mut op_stack = Vec::new();
        let mut getters = HashMap::from_iter(external.iter().map(|&v| (v, 0)));
        let mut stk = InsnSeqStackification {
            inputs: vec![],
            manips: ti_vec![StackManipulators {
                captures: 0,
                getters: vec![],
                inputs: smallvec![],
                outputs: smallvec![],
            }; insns.len() + 1],
        };

        for (i, (inputs, outputs)) in insns.iter().enumerate() {
            stackify_single(
                &mut op_stack,
                &mut getters,
                &mut stk.manips,
                i,
                inputs.to_smallvec(),
                outputs.to_smallvec(),
            );
        }
        remove_excess_operands(
            op_stack.drain(..).zip(std::iter::repeat(false)),
            &mut stk.manips,
            insns.len(),
        );

        stk
    }

    #[test]
    fn ignore_all() {
        let stk = test_skeleton(
            &[],
            &test_insns![
                [] => ["a"],
                [] => ["b"],
                [] => ["c"],
                [] => ["d"],
            ],
        );
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => ["a"];
                [1] cap(1) get[] [] => ["b"];
                [2] cap(1) get[] [] => ["c"];
                [3] cap(1) get[] [] => ["d"];
                [4] cap(1) get[] [] => [];
            }
        );
    }

    #[test]
    fn use_all_in_order() {
        let stk = test_skeleton(
            &[],
            &test_insns![
                [] => ["a"],
                [] => ["b"],
                [] => ["c"],
                [] => ["d"],
                ["a", "b", "c", "d"] => ["e"],
            ],
        );
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => ["a"];
                [1] cap(0) get[] [] => ["b"];
                [2] cap(0) get[] [] => ["c"];
                [3] cap(0) get[] [] => ["d"];
                [4] cap(0) get[] ["a", "b", "c", "d"] => ["e"];
                [5] cap(1) get[] [] => [];
            }
        );
    }

    #[test]
    fn use_skipping_in_order() {
        let stk = test_skeleton(
            &[],
            &test_insns![
                [] => ["a"],
                [] => ["b"],
                [] => ["c"],
                [] => ["d"],
                ["b", "d"] => ["e"],
            ],
        );
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => ["a"];
                [1] cap(1) get[] [] => ["b"];
                [2] cap(0) get[] [] => ["c"];
                [3] cap(1) get[] [] => ["d"];
                [4] cap(0) get[] ["b", "d"] => ["e"];
                [5] cap(1) get[] [] => [];
            }
        );
    }

    #[test]
    fn use_out_of_order() {
        let stk = test_skeleton(
            &[],
            &test_insns![
                [] => ["a"],
                [] => ["b"],
                [] => ["c"],
                [] => ["d"],
                ["b", "d", "c", "a"] => ["e"],
            ],
        );
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => ["a"];
                [1] cap(1) get[] [] => ["b"];
                [2] cap(0) get[] [] => ["c"];
                [3] cap(1) get[] [] => ["d"];
                [4] cap(0) get["c", "a"] ["b", "d", "c", "a"] => ["e"];
                [5] cap(1) get[] [] => [];
            }
        );
    }

    #[test]
    fn use_repeated() {
        let stk = test_skeleton(
            &[],
            &test_insns![
                [] => ["a"],
                [] => ["b"],
                ["a", "a", "b"] => ["c"],
            ],
        );
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => ["a"];
                [1] cap(0) get["a"] [] => ["b"];
                [2] cap(0) get[] ["a", "a", "b"] => ["c"];
                [3] cap(1) get[] [] => [];
            }
        );
    }

    #[test]
    fn queueing_getters() {
        let stk = test_skeleton(
            &[],
            &test_insns![
                [] => ["a"],
                [] => ["b"],
                ["a", "b"] => ["c"],
                [] => ["d"],
                ["b", "a", "d"] => ["e"],
            ],
        );
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => ["a"];
                [1] cap(0) get[] [] => ["b"];
                [2] cap(0) get[] ["a", "b"] => ["c"];
                [3] cap(1) get["b", "a"] [] => ["d"];
                [4] cap(0) get[] ["b", "a", "d"] => ["e"];
                [5] cap(1) get[] [] => [];
            }
        );
    }

    #[test]
    fn staircase() {
        let stk = test_skeleton(
            &[],
            &test_insns![
                [] => ["a"],
                ["a"] => ["b"],
                ["a", "b"] => ["c"],
                ["a", "b", "b", "c"] => ["d"],
            ],
        );
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => ["a"];
                [1] cap(0) get[] ["a"] => ["b"];
                [2] cap(1) get["a", "b", "b", "a", "b"] ["a", "b"] => ["c"];
                [3] cap(0) get[] ["a", "b", "b", "c"] => ["d"];
                [4] cap(1) get[] [] => [];
            }
        );
    }

    #[test]
    fn nested_operators() {
        let stk = test_skeleton(
            &[],
            &test_insns![
                [] => ["a"],
                [] => ["b"],
                ["a", "b"] => ["c"],
                ["b"] => ["d"],
                [] => ["e"],
                ["e"] => ["f"],
                ["c", "f"] => ["g"],
                ["a", "g"] => ["h"],
            ],
        );
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => ["a"];
                [1] cap(0) get[] [] => ["b"];
                [2] cap(0) get[] ["a", "b"] => ["c"];
                [3] cap(0) get["b"] ["b"] => ["d"];
                [4] cap(1) get[] [] => ["e"];
                [5] cap(0) get[] ["e"] => ["f"];
                [6] cap(0) get[] ["c", "f"] => ["g"];
                [7] cap(1) get["a", "g"] ["a", "g"] => ["h"];
                [8] cap(1) get[] [] => [];
            }
        );
    }

    #[test]
    fn external_instructions() {
        let stk = test_skeleton(
            &["ex0"],
            &test_insns![
                ["ex0"] => ["a"],
                ["a"] => ["b"],
                ["b", "a"] => ["c"],
            ],
        );
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get["ex0"] ["ex0"] => ["a"];
                [1] cap(0) get[] ["a"] => ["b"];
                [2] cap(0) get["a"] ["b", "a"] => ["c"];
                [3] cap(1) get[] [] => [];
            }
        )
    }

    #[test]
    fn conflicting_getter_locations() {
        let stk = test_skeleton(
            &[],
            &test_insns![
                [] => ["a"],
                [] => ["b"],
                ["a", "b"] => ["c"],
                ["a"] => ["d"],
                ["c", "b", "d"] => ["e"],
            ],
        );
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => ["a"];
                [1] cap(0) get[] [] => ["b"];
                [2] cap(0) get[] ["a", "b"] => ["c"];
                [3] cap(0) get["b", "a"] ["a"] => ["d"];
                [4] cap(0) get[] ["c", "b", "d"] => ["e"];
                [5] cap(1) get[] [] => [];
            }
        )
    }

    #[test]
    fn getter_before_calculation_after_release() {
        let stk = test_skeleton(
            &[],
            &test_insns![
                [] => ["a"],
                [] => ["b"],
                [] => ["c"],
                ["b", "c"] => ["d"],
                ["d"] => ["e"],
                ["a", "d"] => ["f"],
            ],
        );
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => ["a"];
                [1] cap(0) get[] [] => ["b"];
                [2] cap(0) get[] [] => ["c"];
                [3] cap(0) get[] ["b", "c"] => ["d"];
                [4] cap(0) get[] ["d"] => ["e"];
                [5] cap(1) get["d"] ["a", "d"] => ["f"];
                [6] cap(1) get[] [] => [];
            }
        );
    }

    #[test]
    fn getter_before_calculation_before_release() {
        let stk = test_skeleton(
            &[],
            &test_insns![
                [] => ["a"],
                [] => ["b"],
                [] => ["c"],
                ["b", "c"] => ["d"],
                ["d"] => ["e"],
                ["d", "a"] => ["f"],
            ],
        );
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => ["a"];
                [1] cap(1) get[] [] => ["b"];
                [2] cap(0) get[] [] => ["c"];
                [3] cap(0) get[] ["b", "c"] => ["d"];
                [4] cap(0) get[] ["d"] => ["e"];
                [5] cap(1) get["d", "a"] ["d", "a"] => ["f"];
                [6] cap(1) get[] [] => [];
            }
        );
    }

    #[test]
    fn transparent_instruction() {
        let stk = test_skeleton(
            &[],
            &test_insns![
                [] => ["a"],
                [] => ["b"],
                [] => ["c"],
                [] => ["d"],
                [] => ["e"],
                ["b", "c", "d", "e"] => ["b", "c", "d"],
                ["a", "c"] => ["f"],
                ["d", "b"] => ["g"],
            ],
        );
        assert_eq!(
            stk,
            stackification! {
                inputs [];
                [0] cap(0) get[] [] => ["a"];
                [1] cap(0) get[] [] => ["b"];
                [2] cap(0) get[] [] => ["c"];
                [3] cap(0) get[] [] => ["d"];
                [4] cap(0) get[] [] => ["e"];
                [5] cap(0) get[] ["b", "c", "d", "e"] => ["b", "c", "d"];
                [6] cap(3) get["c"] ["a", "c"] => ["f"];
                [7] cap(1) get["d", "b"] ["d", "b"] => ["g"];
                [8] cap(1) get[] [] => [];
            }
        );
    }
}
