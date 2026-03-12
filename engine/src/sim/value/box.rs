use std::sync::Arc;

use derive_more::derive::{Deref, DerefMut};

use crate::{
    mir::reflection::{MirReflect, MirType, MirTypeContents, MirTypeInfo},
    util::reflection::{Reflect, TypeInfo},
};

#[derive(Deref, DerefMut)]
#[repr(transparent)]
pub struct NlBox<T: Sized>(Box<T>);

impl<T> NlBox<T> {
    pub fn new(value: T) -> Self {
        Self(Box::new(value))
    }
}

unsafe impl<T: Reflect + 'static> Reflect for NlBox<T> {
    const TYPE_INFO: TypeInfo = TypeInfo::new_drop::<NlBox<T>>("NlBox<T>");
}

unsafe impl<T: Reflect + MirReflect + 'static> MirReflect for NlBox<T> {
    fn mir_type() -> MirType {
        Arc::new(MirTypeInfo {
            static_ty: Some(&<NlBox<T>>::TYPE_INFO),
            contents: MirTypeContents::IsPointerTo(T::mir_type()),
        })
    }
}
