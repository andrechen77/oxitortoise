use derive_more::derive::{Deref, DerefMut};
use macro_reflect::{MirReflect, reflect};

use crate::sim::value::NlList;

#[derive(Deref, DerefMut, Clone, MirReflect)]
#[repr(transparent)]
pub struct NlBox<T: Sized>(Box<T>);

#[reflect(clone(dynamic))]
impl Reflect for NlBox<NlList> {}

#[reflect]
impl Reflect for &NlBox<NlList> {}

impl<T> NlBox<T> {
    pub fn new(value: T) -> Self {
        Self(Box::new(value))
    }
}
