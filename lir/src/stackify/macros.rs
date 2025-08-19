pub use crate::stackification;

#[macro_export]
macro_rules! stackification {
    (
        inputs [$($inputs:expr),*];
        $([$pc:expr] cap($captures:expr) get[$($getters:expr),*] [$($insn_inputs:expr),*] => [$($insn_outputs:expr),*];)* $(;)?
    ) => {
        {
            let inputs = vec![$($inputs),*];

            let manips = typed_index_collections::ti_vec![$({
                // let pc = $pc;
                let captures = $captures;
                let getters = vec![$($getters),*];
                let insn_inputs = smallvec![$($insn_inputs),*];
                let insn_outputs = smallvec![$($insn_outputs),*];

                $crate::stackify::StackManipulators {
                    captures,
                    getters,
                    inputs: insn_inputs,
                    outputs: insn_outputs,
                }
            }),*];

            $crate::stackify::InsnSeqStackification {
                inputs,
                manips,
            }
        }
    };
}
