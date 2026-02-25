use derive_more::derive::{Deref, DerefMut};

use crate::util::reflection::{MemRepr, Reflect, TypeInfo};

#[derive(Deref, DerefMut)]
#[repr(transparent)]
pub struct NlBox<T: Sized>(Box<T>);

impl<T> NlBox<T> {
    pub fn new(value: T) -> Self {
        Self(Box::new(value))
    }
}

unsafe impl<T: Reflect + 'static> Reflect for NlBox<T> {
    const TYPE_INFO: TypeInfo =
        TypeInfo::new_drop::<NlBox<T>>("NlBox<T>", MemRepr::Single(lir::ValType::Ptr));
}
