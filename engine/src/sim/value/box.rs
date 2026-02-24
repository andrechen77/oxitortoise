use derive_more::derive::{Deref, DerefMut};

use crate::util::reflection::{MemRepr, TypeInfo, TypeInfoOptions};

#[derive(Deref, DerefMut)]
#[repr(transparent)]
pub struct NlBox<T: Sized>(Box<T>);

impl<T> NlBox<T> {
    pub fn new(value: T) -> Self {
        Self(Box::new(value))
    }
}

pub fn generate_box_type_info<T: 'static>() -> TypeInfo {
    // it is important that the generated TypeInfo applies to every
    // possible type T
    TypeInfo::new_drop::<NlBox<T>>(TypeInfoOptions {
        is_zeroable: false,
        mem_repr: Some(MemRepr::Single(lir::ValType::Ptr)),
    })
}
