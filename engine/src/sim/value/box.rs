use derive_more::derive::{Deref, DerefMut};
use macro_reflect::{ReflectComponents, reflect};

use crate::sim::value::NlList;

#[derive(Deref, DerefMut, Clone, ReflectComponents)]
#[repr(transparent)]
pub struct NlBox<T: Sized>(Box<T>);

#[reflect(clone(dynamic))]
impl Reflect for NlBox<NlList> {}

impl<T> NlBox<T> {
    pub fn new(value: T) -> Self {
        Self(Box::new(value))
    }
}
