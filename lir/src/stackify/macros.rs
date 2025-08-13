use std::collections::BTreeMap;

pub use crate::stackification;
use crate::stackify::{InsnTreeInfo, OutputMode, Stackification};

#[macro_export]
macro_rules! stackification_line {
    ($stackification:expr; $pc_ty:ty; [($pc:expr) =| $released_to:expr]) => {
        $stackification.forest[<$pc_ty>::from($pc)].output_mode =
            OutputMode::Release { parent: <$pc_ty>::from($released_to) };
        $stackification.forest[<$pc_ty>::from($released_to)].subtree_start =
            ($stackification.forest[<$pc_ty>::from($pc)].subtree_start)
                .min($stackification.forest[<$pc_ty>::from($released_to)].subtree_start);
    };
    ($stackification:expr; $pc_ty:ty; [($pc:expr) =* $captured_by:expr]) => {
        $stackification.forest[<$pc_ty>::from($pc)].output_mode =
            OutputMode::Capture { parent: <$pc_ty>::from($captured_by) };
    };
    ($stackification:expr; $pc_ty:ty; [($pc:expr) ==]) => {
        $stackification.forest[<$pc_ty>::from($pc)].output_mode = OutputMode::Available;
    };
    ($stackification:expr; $pc_ty:ty; [($pc:expr) <~ $value:expr]) => {
        $stackification
            .getters
            .entry(<$pc_ty>::from($pc))
            .or_default()
            .push_back(<$pc_ty>::from($value));
    };
}

#[macro_export]
macro_rules! stackification {
    ($stackty:ty; $pc_ty:ty; count $count:expr; $($line:tt)*) => {
        {
            let mut stackification: $stackty = Stackification {
                forest: (0..$count)
                    .map(|pc| InsnTreeInfo {
                        output_mode: OutputMode::Available,
                        subtree_start: <$pc_ty>::from(pc),
                    })
                    .collect(),
                getters: BTreeMap::new(),
            };

            $(
                $crate::stackification_line!(stackification; $pc_ty; $line);
            )*

            stackification
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stackification_macro() {
        let stackification = stackification! {
            Stackification<usize, Vec<InsnTreeInfo<usize>>>;
            usize;
            count 5usize;

            [(0usize) =| 2usize]
            [(1usize) =| 2usize]
            [(2usize) =| 4usize]
            [(3usize) =* 4usize]
            [(4usize) <~ 1usize]
            [(4usize) ==]
        };

        assert_eq!(stackification.forest, vec![
            InsnTreeInfo { output_mode: OutputMode::Release { parent: 2 }, subtree_start: 0 },
            InsnTreeInfo { output_mode: OutputMode::Release { parent: 2 }, subtree_start: 1 },
            InsnTreeInfo { output_mode: OutputMode::Release { parent: 4 }, subtree_start: 0 },
            InsnTreeInfo { output_mode: OutputMode::Capture { parent: 4 }, subtree_start: 3 },
            InsnTreeInfo { output_mode: OutputMode::Available, subtree_start: 0 },
        ]);
        assert_eq!(stackification.getters, BTreeMap::from([(4, vec![1].into())]));
    }
}
