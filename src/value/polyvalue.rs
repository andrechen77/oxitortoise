//! A container that can hold any of the possible types of values in NetLogo,
//! with optional type safety.
//!
//! A `PolyValue` consists of some data, which can be of various NetLogo runtime
//! type (see [`super`], as well as a type tag, which indicates the type of the
//! data. However, unlike regular Rust enums or tagged unions, there is an
//! additional type tag, [`Type::Erased`], which indicates that the type of the
//! item is untracked at runtime. This requires the user to manually remember
//! what the type of the value was, so that the correct type can be retrieved
//! later.
//!
//! A `PolyValue` is considered either "uninitalized" (tagged with [`Type::Uninit`]), "type-erased" (tagged with
//! [`Type::Erased`]), or "type-checked" (tagged with an actual type). All
//! initialized `PolyValues` with inner values that implement `Drop` must manually drop their inner value before the
//! entire `PolyValue` itself is dropped, including when assigning to the entire
//! `PolyValue` (there is a check for this in the debug build).
//!
//! A type-checked `PolyValue` is like a Rust enum, with a tag that indicates
//! which variant is stored. This module guarantees the validity of the tag
//! and the stored type for type-checked `PolyValue`s. The inner value of a
//! type-checked `PolyValue` can be safely retrieved using using the [`PolyValue::get`]
//! and [`PolyValue::get_mut`] methods, or unsafely using the [`PolyValue::get_unchecked`]
//! and [`PolyValue::get_mut_unchecked`] methods. The inner value can be
//! safely dropped using the [`PolyValue::drop_inner`] method, or unsafely using the
//! [`PolyValue::drop_inner_unchecked`] method.
//!
//! A type-erased `PolyValue` provides no runtime checks for accessing the
//! correct data. All the safe accessors will fail (i.e. return `None`) with
//! type-erased values. The only way to access the value is with unsafe methods
//! (with names ending in `_unchecked`). Type-erased `PolyValue`s can be unsafely
//! dropped using [`PolyValue::drop_inner_unchecked`].

use std::{fmt::Debug, mem::ManuallyDrop};

use crate::{patch::PatchId, turtle::TurtleId};

use super::{Boolean, Float, String};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Type {
    /// Indicates that the type of the value is unknown. The only way to get the
    /// value is to unsafely read from the union, which should only be done
    /// if the caller is confident that they are reading the correct type.
    Erased,
    /// Indicates that there is no data stored in the value. Dropping a
    /// [`PolyValue`] with this type does nothing.
    Uninit,
    Float,
    Boolean,
    String,
    Nobody,
    Turtle,
    Patch,
}

impl Type {
    /// Returns whether the type needs to run a destructor when dropped; i.e.
    /// whether the associated type implements [`Drop`].
    const fn needs_drop(&self) -> bool {
        match self {
            Type::Erased => true,
            Type::Uninit => false,
            Type::Float | Type::Boolean | Type::Nobody | Type::Turtle | Type::Patch => false,
            Type::String => true,
        }
    }
}

union UntypedData {
    unit: (),
    float: Float,
    boolean: Boolean,
    string: ManuallyDrop<String>,
    turtle: TurtleId,
    patch: PatchId,
}

pub struct PolyValue {
    /// The type of the data. If it is not erased, then the data field is
    /// guaranteed to be of the specified type.
    r#type: Type,
    data: UntypedData,
}

impl PolyValue {
    pub const NOBODY: Self = Self {
        r#type: Type::Nobody,
        data: UntypedData { unit: () },
    };

    // SAFETY: much of this code relies on the equality operator to check the
    // tag of the value and make sure that it has the correct type, which is not
    // technically allowed. See
    // https://doc.rust-lang.org/std/cmp/trait.PartialEq.html. However, as long
    // as the implementation of PartialEq for Type is actually correct, this
    // will be correct.

    /// Asserts that the value is of a certain type. This should only be called
    /// when the value is initialized to the specified type.
    pub unsafe fn assert_type(&mut self, r#type: Type) {
        if self.r#type == r#type {
            return;
        }
        debug_assert_eq!(
            self.r#type,
            Type::Erased,
            "cannot assert the type of a value that is already known to have a different type"
        );
        self.r#type = r#type;
    }

    pub fn get<T: ContainedInValue>(&self) -> Option<&T> {
        if self.r#type == T::TYPE_TAG {
            // SAFETY: type tag ensures valid union access
            Some(unsafe { self.get_unchecked()})
        } else {
            None
        }
    }

    pub fn get_mut<T: ContainedInValue>(&mut self) -> Option<&mut T> {
        if self.r#type == T::TYPE_TAG {
            // SAFETY: type tag ensures valid union access
            Some(unsafe { self.get_mut_unchecked()})
        } else {
            None
        }
    }

    pub fn into<T: ContainedInValue>(self) -> Option<T> {
        if self.r#type == T::TYPE_TAG {
            // SAFETY: type tag ensures valid union access
            Some(unsafe { self.into_unchecked()})
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// The value must actually contain the specified type.
    pub unsafe fn get_unchecked<T: ContainedInValue>(&self) -> &T {
        // This cast is valid because it is between types with identical layouts
        // and bit validities.
        let ptr: *const T = T::location_in(self).cast();
        // SAFETY: preconditioned that the value actually holds this type
        unsafe { &*ptr }
    }

    /// # Safety
    ///
    /// The value must actually contain the specified type. Use assignment
    /// instead of this method to initialize the type from an uninitialized
    /// state or to change the type.
    pub unsafe fn get_mut_unchecked<T: ContainedInValue>(&mut self) -> &mut T {
        // This cast is valid because it is between types with identical layouts
        // and bit validities.
        let ptr: *mut T = T::location_in_mut(self).cast();
        // SAFETY: preconditioned that the value actually holds this type
        unsafe { &mut *ptr }
    }

    /// # Safety
    ///
    /// The value must actually contain the specified type.
    pub unsafe fn into_unchecked<T: ContainedInValue>(self) -> T {
        // prevent the destructor from running
        let value = ManuallyDrop::new(self);
        // SAFETY: value is an in-bounds, valid allocation
        let ptr: *const T = unsafe { T::location_in(&*value) }.cast();
        // SAFETY: preconditioned that the value actually holds this type
        unsafe { ptr.read() }
    }

    /// Safely runs the destructor for the inner value.
    ///
    /// # Panics
    ///
    /// Panics if the value is uninitialized or type-erased.
    pub fn drop_inner(&mut self) {
        // set the type to uninit up-front for exception safety reasons
        let old_type = self.r#type;
        self.r#type = Type::Uninit;
        match old_type {
            Type::Erased => panic!("cannot safely drop a type-erased value"),
            Type::Uninit => panic!("cannot drop an uninitialized value"),
            Type::Float | Type::Boolean | Type::Nobody | Type::Patch | Type::Turtle => {}
            Type::String => {
                // SAFETY: type tag ensures valid union access. the data is not
                // used again because this is the destructor
                unsafe { ManuallyDrop::drop(&mut self.data.string) };
            }
        }
    }

    /// Runs the destructor for the inner type stored in the value.
    ///
    /// # Safety
    ///
    /// The value must actually contain the specified type.
    pub unsafe fn drop_inner_unchecked<T: ContainedInValue>(&mut self) {
        debug_assert!(self.r#type == Type::Erased || self.r#type == T::TYPE_TAG);

        self.r#type = Type::Uninit;

        // SAFETY: value is an in-bounds, valid allocation
        let ptr: *const T = unsafe { T::location_in(&*self) }.cast();
        // SAFETY: preconditioned that the value actually holds this type
        let inner = unsafe { ptr.read() };
        drop(inner);
    }

    /// Resets the value to its default value.
    /// TODO should this always be zero?
    pub fn reset(&mut self) {
        *self = PolyValue::default();
    }
}

/// A data type that is contained inside a [`PolyValue`].
///
/// # Safety
///
/// This trait is unsafe to implement because it triggers methods for conversions
/// between `Self` and [`PolyValue`]. To be safe, it must be able to guarantee that
/// the `location_in` method always returns a pointer that is valid, either if
/// the [`Type`] tag of the [`PolyValue`] matches `TYPE_TAG`, or if the data in the
/// value was last stored as a valid instance of `Self`.
pub unsafe trait ContainedInValue {
    const TYPE_TAG: Type;

    /// # Safety
    ///
    /// Must be called with an in-bounds pointer.
    unsafe fn location_in(value: *const PolyValue) -> *const Self;

    /// # Safety
    ///
    /// Must be called with an in-bounds pointer.
    unsafe fn location_in_mut(value: *mut PolyValue) -> *mut Self;
}

impl Default for PolyValue {
    fn default() -> Self {
        PolyValue {
            r#type: Type::Float,
            data: UntypedData {
                float: Float::new(0.0),
            },
        }
    }
}

// TODO nobody semantics: a dead turtle ID compares equal to nobody
impl PartialEq for PolyValue {
    fn eq(&self, other: &Self) -> bool {
        // this macro must only be called if you are sure that the values are in
        // the specified field for both self and other
        macro_rules! compare_by_field {
            ($field:ident) => {{
                // SAFETY: type tag ensures valid union access
                let lhs = unsafe { &self.data.$field };
                let rhs = unsafe { &other.data.$field };
                lhs == rhs
            }};
        }
        match (self.r#type, other.r#type) {
            (Type::Erased, _) | (_, Type::Erased) => panic!("cannot compare type-erased value"),
            (Type::Uninit, _) | (_, Type::Uninit) => panic!("cannot compare uninitialized value"),
            (Type::Float, Type::Float) => compare_by_field!(float),
            (Type::Boolean, Type::Boolean) => compare_by_field!(boolean),
            (Type::String, Type::String) => compare_by_field!(string),
            (Type::Nobody, Type::Nobody) => true,
            (Type::Turtle, Type::Turtle) => compare_by_field!(turtle),
            (Type::Patch, Type::Patch) => compare_by_field!(patch),
            _ => todo!(),
        }
    }
}

impl Debug for PolyValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // SAFETY: type tag ensures valid union access
        match self.r#type {
            Type::Erased => write!(f, "Value::Erased"),
            Type::Uninit => write!(f, "Value::Uninit"),
            Type::Float => write!(f, "Value::Float({:?})", unsafe { self.data.float }),
            Type::Boolean => write!(f, "Value::Boolean({:?})", unsafe { self.data.boolean }),
            Type::String => write!(f, "Value::String({:?})", unsafe { &self.data.string }),
            Type::Nobody => write!(f, "Value::Nobody"),
            Type::Turtle => write!(f, "Value::Turtle({:?})", unsafe { self.data.turtle }),
            Type::Patch => write!(f, "Value::Patch({:?})", unsafe { self.data.patch }),
        }
    }
}

impl Drop for PolyValue {
    fn drop(&mut self) {
        debug_assert!(!self.r#type.needs_drop());
    }
}

impl<T: ContainedInValue> From<T> for PolyValue {
    fn from(inner: T) -> Self {
        let mut value = PolyValue { r#type: T::TYPE_TAG, data: UntypedData { unit: () } };

        // SAFETY: the pointer was created to an in-bounds allocation
        let data_ptr = unsafe { T::location_in_mut(&raw mut value) };

        // SAFETY: both pointers point to a valid allocation, location_in_mut
        // guarantees a valid *mut T if the input *mut Value is valid
        unsafe { data_ptr.write(inner) }

        value
    }
}

/// Implements infallible conversion from subtype to [`PolyValue`], fallible
/// conversions from [`PolyValue`] to subtype, and unchecked conversions from
/// [`PolyValue`] to subtype. To be safe, invocations of the macro must guarantee
/// that for any [`PolyValue`] `v`, `v.data.$union_field` has some type that is
/// bitwise identical to `$type`, and that that `v.data.$union_field` field is
/// valid, either if `v.r#type` of the [`PolyValue`] matches `$type_tag`, or if the
/// data in the value was last stored as a valid instance of `Self`.
macro_rules! impl_conv {
    ($type:ty, $type_tag:expr, $union_field:ident) => {
        // SAFETY:
        unsafe impl ContainedInValue for $type {
            const TYPE_TAG: Type = $type_tag;

            unsafe fn location_in(value: *const PolyValue) -> *const Self {
                // SAFETY: we are not accessing the value, just computing the
                // projection. the projection is known to be in bounds because
                // it came from a reference. The cast retains the validity of
                // the pointer because the cast is between types with the same
                // layout and bit validity (i.e. T and ManuallyDrop<T>)
                unsafe { &raw const (*value).data.$union_field }.cast()

            }

            unsafe fn location_in_mut(value: *mut PolyValue) -> *mut Self {
                // SAFETY: we are not accessing the value, just computing the
                // projection. the projection is known to be in bounds because
                // it came from a reference. The cast retains the validity of
                // the pointer because the cast is between types with the same
                // layout and bit validity (i.e. T and ManuallyDrop<T>)
                unsafe { &raw mut (*value).data.$union_field }.cast()
            }
        }
    };
}

impl_conv!(Boolean, Type::Boolean, boolean);
impl_conv!(Float, Type::Float, float);
impl_conv!(TurtleId, Type::Turtle, turtle);
impl_conv!(PatchId, Type::Patch, patch);
impl_conv!(String, Type::String, string);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_test() {
        let mut value: PolyValue = String::new().into();
        value.drop_inner(); // string needs to be dropped
        value = Boolean(true).into();
        // boolean does not need to be dropped
        drop(value);
    }

    #[test]
    #[should_panic]
    fn cannot_forget_to_drop() {
        #[allow(unused_assignments)]
        let mut value: PolyValue = String::new().into();
        value = Boolean(true).into(); // panics here
        drop(value);
    }

    #[test]
    fn drop_test_erased() {
        let mut value: PolyValue = String::new().into();
        value.r#type = Type::Erased;
        unsafe { value.drop_inner_unchecked::<String>() };
        drop(value);
    }

    #[test]
    fn store_retrieve_safely() {
        let value = PolyValue::from(Float::new(3.14));
        assert_eq!(value.r#type, Type::Float);
        assert_eq!(value.get::<Float>().unwrap().get(), 3.14);
    }

    #[test]
    fn store_retrieve_unsafely() {
        let value = PolyValue::from(Float::new(3.14));
        assert_eq!(unsafe { value.get_unchecked::<Float>() }.get(), 3.14);
    }

    #[test]
    fn nobody_equality() {
        assert_eq!(PolyValue::NOBODY, PolyValue::NOBODY);
    }

    #[test]
    fn test_get_unchecked() {
        let value = PolyValue::from(Float::new(3.14));
        assert_eq!(unsafe { value.get_unchecked::<Float>() }.get(), 3.14);
    }

    #[test]
    fn test_get_mut_unchecked() {
        let mut value = PolyValue::from(Float::new(3.14));
        assert_eq!(unsafe { value.get_mut_unchecked::<Float>() }.get(), 3.14);
        *unsafe { value.get_mut_unchecked::<Float>() } = Float::new(2.71);
        assert_eq!(unsafe { value.get_unchecked::<Float>() }.get(), 2.71);
    }

    #[test]
    fn test_into() {
        let value = PolyValue::from(Float::new(3.14));
        assert_eq!(value.into(), Some(Float::new(3.14)));
    }

    #[test]
    fn test_into_unchecked() {
        let value = PolyValue::from(Float::new(3.14));
        let float: Float = unsafe { value.into_unchecked() };
        assert_eq!(float.get(), 3.14);
    }

    #[test]
    fn test_store_as_float_retrieve_as_boolean() {
        let value = PolyValue::from(Float::new(3.14));
        assert!(value.get::<Boolean>().is_none());
    }
}

