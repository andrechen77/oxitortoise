use std::{fmt::Debug, mem::ManuallyDrop};

use derive_more::derive::{From, Into};

use crate::sim::{color::Color, patch::PatchId, turtle::TurtleId, value::{Boolean, Float, Nobody, String, TryAsFloat}};

/// A tag used to indicate the type of of a dynamically-typed value located
/// somewhere else.
#[derive(Debug, Clone, Copy, From, Into, PartialEq, Eq)]
#[repr(transparent)]
pub struct TypeId(pub u8);

impl TypeId {
    pub const UNINIT: TypeId = TypeId(0);
    pub const FLOAT: TypeId = TypeId(1);
    pub const COLOR: TypeId = TypeId(2);
    pub const BOOLEAN: TypeId = TypeId(3);
    pub const STRING: TypeId = TypeId(4);
    pub const NOBODY: TypeId = TypeId(5);
    pub const TURTLE: TypeId = TypeId(6);
    pub const PATCH: TypeId = TypeId(7);
}

pub union UntypedVal {
    pub unit: (),
    pub float: ManuallyDrop<Float>,
    pub color: ManuallyDrop<Color>,
    pub boolean: ManuallyDrop<Boolean>,
    pub string: ManuallyDrop<String>,
    pub nobody: ManuallyDrop<Nobody>,
    pub turtle: ManuallyDrop<TurtleId>,
    pub patch: ManuallyDrop<PatchId>,
    // TODO ensure the union is big enough to fit types that extensions might
    // want to put (i.e. 8 bytes).
}

/// Indicates that a `Self` reference can be obtained from a `UntypedVal`
/// reference, if `UntypedVal` actually contains the correct type.
pub unsafe trait InUntypedVal: Debug + TryAsFloat {
    /// # Safety
    ///
    /// Callers must ensure that the passed-in `TypeId` actually corresponds to
    /// the type of the `UntypedVal` passed in.
    ///
    /// Implementors must ensure that `Some` is only returned if the
    /// `UntypedVal` actually contains `Self`, which implies no `TypeId`
    /// collisions among different types.
    unsafe fn from_untyped(type_id: TypeId, untyped: &UntypedVal) -> Option<&ManuallyDrop<Self>> where Self: Sized;

    /// # Safety
    ///
    /// Callers must ensure that the passed-in `TypeId` actually corresponds to
    /// the type of the `UntypedVal` passed in.
    ///
    /// Implementors must ensure that `Some` is only returned if the
    /// `UntypedVal` actually contains `Self`, which implies no `TypeId`
    /// collisions among different types.
    unsafe fn from_untyped_mut(type_id: TypeId, untyped: &mut UntypedVal) -> Option<&mut ManuallyDrop<Self>> where Self: Sized;

    /// # Safety
    ///
    /// Implementations must ensure the returned `TypeId` must match the
    /// returned `UntypedVal`, such that it will be successfully retrieved.
    fn clone_to_untyped(&self) -> (TypeId, UntypedVal);
}

macro_rules! impl_from_untyped {
    ($type:ty, $type_tag:expr, $union_field:ident) => {
        unsafe impl InUntypedVal for $type {
            unsafe fn from_untyped(type_id: TypeId, untyped: &UntypedVal) -> Option<&ManuallyDrop<Self>> {
                if type_id == $type_tag {
                    // SAFETY: it is a precondition of this function that the
                    // union actually contains `Self` if the type ID matches
                    Some(unsafe { &untyped.$union_field })
                } else {
                    None
                }
            }

            unsafe fn from_untyped_mut(type_id: TypeId, untyped: &mut UntypedVal) -> Option<&mut ManuallyDrop<Self>> {
                if type_id == $type_tag {
                    // SAFETY: it is a precondition of this function that the
                    // union actually contains `Self` if the type ID matches
                    Some(unsafe { &mut untyped.$union_field })
                } else {
                    None
                }
            }

            fn clone_to_untyped(&self) -> (TypeId, UntypedVal) {
                ($type_tag, UntypedVal { $union_field: ManuallyDrop::new(self.clone()) })
            }
        }
    };
}

impl_from_untyped!(Boolean, TypeId::BOOLEAN, boolean);
impl_from_untyped!(Float, TypeId::FLOAT, float);
impl_from_untyped!(Color, TypeId::COLOR, color);
impl_from_untyped!(Nobody, TypeId::NOBODY, nobody);
impl_from_untyped!(TurtleId, TypeId::TURTLE, turtle);
impl_from_untyped!(PatchId, TypeId::PATCH, patch);
impl_from_untyped!(String, TypeId::STRING, string);
