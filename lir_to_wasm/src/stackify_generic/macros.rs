pub use crate::stackification;

#[macro_export]
macro_rules! stackification {
    (
        inputs [$($inputs:expr),*];
        $([$idx:expr] cap($captures:expr) get[$($getters:expr),*];)* $(;)?
    ) => {
        {
            let inputs = vec![$($inputs),*];

            let manips = $crate::lir::typed_index_collections::ti_vec![$({
                let captures = $captures;
                let getters = vec![$($getters.into()),*];

                $crate::stackify_generic::StackManipulators {
                    captures,
                    getters,
                }
            }),*];

            $crate::stackify_generic::InsnSeqStackification {
                inputs,
                manips,
            }
        }
    };
}
