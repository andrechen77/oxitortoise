use std::sync::Arc;

use crate::{
    mir::reflection::{MirReflect, MirType, MirTypeInfo},
    util::reflection::{Reflect, TypeInfo},
};

#[derive(Debug, Default, Clone, Copy)]
pub struct Nobody;

unsafe impl Reflect for Nobody {
    const TYPE_INFO: TypeInfo = TypeInfo::new_copy::<Nobody>("Nobody", true);
}

unsafe impl MirReflect for Nobody {
    fn mir_type() -> MirType {
        Arc::new(MirTypeInfo {
            static_ty: Some(&<Nobody>::TYPE_INFO),
            contents: Default::default(),
        })
    }
}
