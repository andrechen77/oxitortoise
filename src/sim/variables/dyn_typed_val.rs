use derive_more::derive::{From, TryInto};

use crate::sim::{
    patch::PatchId,
    turtle::TurtleId,
    value::{Boolean, Float, String},
};

use super::raw_parts::TypeId;

#[derive(Debug, Clone, From, TryInto, PartialEq)]
#[non_exhaustive]
#[repr(C, u8)] // The discriminant must be identical to a TypeId
pub enum DynTypedRef<'a> {
    /// Indicates that there is no data stored in the value. Dropping a
    /// [`PolyValue`] with this type does nothing.
    Uninit = 0,
    Float(&'a Float),
    Boolean(&'a Boolean),
    String(&'a String),
    Nobody,
    Turtle(&'a TurtleId),
    Patch(&'a PatchId),
}

impl DynTypedRef<'_> {
    pub fn get_type(&self) -> TypeId {
        let s: *const TypeId = (self as *const DynTypedRef).cast();
        // SAFETY: because `Self` is marked `repr(C, u8)`, it has a u8 has the
        // first field, which has the same ABI as a TypeId, which is
        // repr(transparent).
        unsafe { *s }
    }
}
