use derive_more::derive::{From, Not};

use crate::mir::reflection::{MemDesc, Reflect, Type, TypeInfo};

#[derive(Debug, Clone, Copy, From, PartialEq, Eq, PartialOrd, Ord, Not)]
#[repr(transparent)]
pub struct NlBool(pub bool);

unsafe impl Reflect for NlBool {
    const TYPE: Type = Type::new(&TypeInfo::new_copy::<NlBool>(
        "NlBool",
        true,
        &MemDesc::IsPrimitive(lir::ValType::I8),
    ));
}
