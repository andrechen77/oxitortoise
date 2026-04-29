use std::{alloc::Layout, ptr::NonNull, sync::LazyLock};

use macro_reflect::reflect;

use crate::{DynType, Reflect, mir::HostFunctionInfo};

// TODO what to do about lifetimes? could cause unsafety and sadness

/// Information about a type that is used by the engine to generate code that
/// manipulates values of the corresponding type.
#[derive(Debug, Clone)]
pub struct StaticTypeInfo {
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
    pub dyn_type: &'static LazyLock<DynType>,
}

impl PartialEq for StaticTypeInfo {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for StaticTypeInfo {}

impl StaticTypeInfo {
    pub fn is<T: Reflect>(&self) -> bool {
        // self.unique_id == Some(Either::Left(std::any::TypeId::of::<T>()))
        self == T::STATIC_TYPE
    }
}

pub type StaticType = &'static StaticTypeInfo;

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

#[reflect(clone(copy), no_drop)]
impl Reflect for () {}

#[reflect(clone(copy), no_drop)]
impl Reflect for bool {}

#[reflect(clone(copy), no_drop)]
impl Reflect for u32 {}

#[reflect(clone(copy), no_drop)]
impl Reflect for f64 {}

#[reflect(clone(copy), no_drop)]
impl Reflect for fn(NonNull<u8>) {}

#[reflect(clone(copy), no_drop)]
impl Reflect for *mut u8 {}
