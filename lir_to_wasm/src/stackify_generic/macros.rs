pub use crate::stackification;

#[macro_export]
macro_rules! stackification {
    (
        inputs [$($inputs:expr),*];
        $([$idx:expr] cap($captures:expr) get[$($getters:expr),*] [$($insn_inputs:expr),*] => [$($insn_outputs:expr),*];)* $(;)?
    ) => {
        {
            let inputs = vec![$($inputs),*];

            let manips = $crate::lir::typed_index_collections::ti_vec![$({
                let captures = $captures;
                let getters = vec![$($getters.into()),*];
                let insn_inputs = smallvec![$($insn_inputs.into()),*];
                let insn_outputs = smallvec![$($insn_outputs.into()),*];

                $crate::stackify_generic::StackManipulators {
                    captures,
                    getters,
                    inputs: insn_inputs,
                    outputs: insn_outputs,
                }
            }),*];

            $crate::stackify_generic::InsnSeqStackification {
                inputs,
                manips,
            }
        }
    };
}
