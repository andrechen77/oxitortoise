use std::sync::LazyLock;

use derive_more::derive::{From, Not};

use crate::util::reflection::{ConcreteTy, MemRepr, Reflect, TypeInfo, TypeInfoOptions};

#[derive(Debug, Clone, Copy, From, PartialEq, Eq, PartialOrd, Ord, Not)]
#[repr(transparent)]
pub struct NlBool(pub bool);

unsafe impl Reflect for NlBool {
    fn ty() -> ConcreteTy {
        static TY: LazyLock<ConcreteTy> = LazyLock::new(|| {
            ConcreteTy::new(&TypeInfo::new_copy::<NlBool>(TypeInfoOptions {
                is_zeroable: true,
                mem_repr: Some(MemRepr::Single(lir::ValType::I8)),
            }))
        });
        TY.clone()
    }
}
