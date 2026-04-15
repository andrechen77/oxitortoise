use std::{alloc::Layout, ptr::NonNull};

use macro_reflect::reflect;

use crate::mir::{HostFunctionInfo, MirType};

// TODO what to do about lifetimes? could cause unsafety and sadness

/// A trait to indicate that the compiler can generate code to manipulate values
/// of this type.
///
/// # Safety
///
/// Implementors must guarantee that the associated `Type` is correct, as
/// the information will be used to generate and run unsafe code.
pub unsafe trait Reflect {
    const TYPE: Type;
}

/// Information about a type that is used by the engine to generate code that
/// manipulates values of the corresponding type.
#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub debug_name: &'static str,
    pub layout: Option<Layout>,
    /// Whether this type is valid at the all-zero bit pattern *and* represents
    /// the numeric value 0.0.
    pub is_zeroable: bool,
    pub clone: CloneKind,
    /// The drop function for this type. As is standard for drop functions, this
    /// should deallocate any memory that the value itself owns, but does not
    /// deallocate the memory that the value itself inhabits (that is the
    /// responsibility of whoever owns the value, i.e. the caller of this
    /// function). None indicates that the type does not need to be dropped.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the passed pointer is a valid pointer to
    /// T that can be dropped, and that that value will never be used again.
    pub drop_fn: Option<unsafe fn(*mut u8)>,
    /// If this type has a special MIR type, with additional information such as
    /// subfields, this function will construct it.
    pub mir_type: Option<fn() -> MirType>,
}

impl PartialEq for TypeInfo {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for TypeInfo {}

impl TypeInfo {
    pub fn is<T: Reflect>(&self) -> bool {
        // self.unique_id == Some(Either::Left(std::any::TypeId::of::<T>()))
        self == T::TYPE
    }
}

pub type Type = &'static TypeInfo;

#[derive(Debug, Clone)]
pub enum CloneKind {
    /// The type can be bitwise copied like Rust's `Copy` types.
    ///
    /// This also mutable references which can be bitwise copied but are not
    /// `Copy`.
    Copy,
    /// The type can be cloned using the specified function.
    Dynamic { clone_fn_info: &'static HostFunctionInfo },
    /// The type cannot be cloned, only moved.
    None,
}

#[reflect(clone(copy))]
impl Reflect for () {}

#[reflect(clone(copy))]
impl Reflect for bool {}

#[reflect(clone(copy))]
impl Reflect for u32 {}

#[reflect(clone(copy))]
impl Reflect for f64 {}

#[reflect(clone(copy))]
impl Reflect for fn(NonNull<u8>) {}

#[reflect(clone(copy))]
impl Reflect for *mut u8 {}
