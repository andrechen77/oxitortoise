use derive_more::derive::{From, Not};

use crate::util::reflection::{MemRepr, Reflect, TypeInfo};

#[derive(Debug, Clone, Copy, From, PartialEq, Eq, PartialOrd, Ord, Not)]
#[repr(transparent)]
pub struct NlBool(pub bool);

unsafe impl Reflect for NlBool {
    const TYPE_INFO: TypeInfo =
        TypeInfo::new_copy::<NlBool>("NlBool", true, MemRepr::Single(lir::ValType::I8));
}
