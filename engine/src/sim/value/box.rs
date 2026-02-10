use derive_more::derive::{Deref, DerefMut};

use crate::util::reflection::{ConstTypeName, TypeInfo, TypeInfoOptions};

#[derive(Deref, DerefMut)]
#[repr(transparent)]
pub struct NlBox<T: Sized>(Box<T>);

impl<T> NlBox<T> {
    pub fn new(value: T) -> Self {
        Self(Box::new(value))
    }
}

pub const fn generate_box_type_info<T: 'static + ConstTypeName>() -> TypeInfo {
    // it is important that the generated TypeInfo applies to every
    // possible type T
    TypeInfo::new::<NlBox<T>>(TypeInfoOptions {
        is_zeroable: false,
        mem_repr: Some(&[(0, lir::MemOpType::Ptr)]),
    })
}

impl<T> ConstTypeName for NlBox<T> {
    const TYPE_NAME: &'static str = "NlBox<?>";
}
