use derive_more::derive::{Deref, DerefMut};

use crate::mir::reflection::{MemDesc, Reflect, Type, TypeInfo};

#[derive(Deref, DerefMut)]
#[repr(transparent)]
pub struct NlBox<T: Sized>(Box<T>);

impl<T> NlBox<T> {
    pub fn new(value: T) -> Self {
        Self(Box::new(value))
    }
}

unsafe impl<T: Reflect + 'static> Reflect for NlBox<T> {
    const TYPE: Type = Type::new(&TypeInfo::new_drop::<NlBox<T>>(
        "NlBox<T>",
        // pointee should be `&T::TYPE.info().mem_desc` but the compiler isn't
        // smart enough to const promote the whole thing
        &MemDesc::StaticIsPointerTo { pointee: &MemDesc::None },
    ));
}
