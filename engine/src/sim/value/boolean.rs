use std::sync::Arc;

use derive_more::derive::{From, Not};

use crate::{
    mir::reflection::{MirReflect, MirType, MirTypeContents, MirTypeInfo},
    util::reflection::{Reflect, TypeInfo},
};

#[derive(Debug, Clone, Copy, From, PartialEq, Eq, PartialOrd, Ord, Not)]
#[repr(transparent)]
pub struct NlBool(pub bool);

unsafe impl Reflect for NlBool {
    const TYPE_INFO: TypeInfo = TypeInfo::new_copy::<NlBool>("NlBool", true);
}

unsafe impl MirReflect for NlBool {
    fn mir_type() -> MirType {
        Arc::new(MirTypeInfo {
            static_ty: Some(&NlBool::TYPE_INFO),
            contents: MirTypeContents::IsPrimitive(lir::ValType::I8),
        })
    }
}
