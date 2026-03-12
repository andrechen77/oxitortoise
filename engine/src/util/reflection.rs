use std::alloc::Layout;

use either::Either;

// TODO what to do about lifetimes? could cause unsafety and sadness

/// A trait to indicate that the compiler can generate code to manipulate values
/// of this type.
///
/// # Safety
///
/// Implementors must guarantee that the associated `Type` is correct, as
/// the information will be used to generate and run unsafe code.
pub unsafe trait Reflect {
    const TYPE_INFO: TypeInfo;
}

// pub fn arc_type_info<T: Reflect + 'static>() -> Arc<TypeInfo> {
//     use std::{
//         collections::HashMap,
//         sync::{LazyLock, RwLock},
//     };
//     // we have to use a single static variable to cover all types because Rust
//     // does not support generic static
//     static TYPE_INFO: LazyLock<RwLock<HashMap<std::any::TypeId, Arc<TypeInfo>>>> =
//         LazyLock::new(|| RwLock::new(HashMap::new()));

//     let key = std::any::TypeId::of::<T>();
//     if let Some(type_info) = TYPE_INFO.read().unwrap().get(&key) {
//         type_info.clone()
//     } else {
//         let type_info = Arc::new(T::TYPE_INFO);
//         TYPE_INFO.write().unwrap().insert(key, type_info.clone());
//         type_info
//     }
// }

/// Information about a type that is used by the engine to generate code that
/// manipulates values of the corresponding type.
#[derive(Debug, Clone)]
pub struct TypeInfo {
    /// An identifier that is always different for types that differ by more
    /// than just lifetimes. This is the only fields used to check for type
    /// identity.
    ///
    /// Between two types that differ only in lifetimes, this will be the same.
    /// For example, `&mut T<'_>` (for type `T` known at compile time) will use
    /// the type id of `&'static mut T<'static>`, even though they are not the
    /// same type.
    ///
    /// Types known at compile time are given a [`std::any::TypeId`], while
    /// types registered at runtime are given a unique integer.
    ///
    /// If the type description is not given a unique id, it cannot be
    /// considered equal to any other type description unless the description is
    /// being compared to itself (in terms of object identity).
    pub unique_id: Option<Either<std::any::TypeId, u32>>,
    pub debug_name: &'static str,
    pub layout: Option<Layout>,
    /// Whether this type is valid at the all-zero bit pattern *and* represents
    /// the numeric value 0.0.
    pub is_zeroable: bool,
    /// The drop function for this type. As is standard for drop functions, this
    /// should deallocate any memory that the value itself owns, but does not
    /// deallocate the memory that the value itself inhabits (that is the
    /// responsibility of whoever owns the value, i.e. the caller of this
    /// function). None indicates that the type is `Copy`.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the passed pointer is a valid pointer to
    /// T that can be dropped, and that that value will never be used again.
    pub drop_fn: Option<unsafe fn(*mut u8)>,
    // /// Information about the contents of the type.
    // pub contents: MemDesc,
}

impl PartialEq for TypeInfo {
    fn eq(&self, other: &Self) -> bool {
        if let (Some(left_id), Some(right_id)) = (self.unique_id, other.unique_id) {
            left_id == right_id
        } else {
            std::ptr::eq(self, other)
        }
    }
}

unsafe fn drop_impl<T>(ptr: *mut u8) {
    unsafe {
        std::ptr::drop_in_place(ptr as *mut T);
    }
}

impl TypeInfo {
    pub fn is<T: 'static>(&self) -> bool {
        self.unique_id == Some(Either::Left(std::any::TypeId::of::<T>()))
    }
}

impl TypeInfo {
    pub const fn new_drop<T: 'static>(debug_name: &'static str) -> Self {
        Self {
            unique_id: Some(Either::Left(std::any::TypeId::of::<T>())),
            debug_name,
            layout: Some(Layout::new::<T>()),
            is_zeroable: false,
            drop_fn: Some(drop_impl::<T>),
        }
    }

    pub const fn new_drop_zeroable<T: 'static>(debug_name: &'static str) -> Self {
        Self {
            unique_id: Some(Either::Left(std::any::TypeId::of::<T>())),
            debug_name,
            layout: Some(Layout::new::<T>()),
            is_zeroable: true,
            drop_fn: Some(drop_impl::<T>),
        }
    }

    pub const fn new_copy<T: Copy + 'static>(debug_name: &'static str, is_zeroable: bool) -> Self {
        Self {
            unique_id: Some(Either::Left(std::any::TypeId::of::<T>())),
            debug_name,
            layout: Some(Layout::new::<T>()),
            is_zeroable,
            drop_fn: None,
        }
    }

    // pub const fn new_mut_ref_to<T: Reflect + 'static>(debug_name: &'static str) -> Self {
    //     Self {
    //         unique_id: Either::Left(std::any::TypeId::of::<&'static mut T>()),
    //         debug_name,
    //         layout: Some(Layout::new::<&mut T>()),
    //         is_zeroable: false,
    //         drop_fn: None, // mut refs have no destructor, so this is correct
    //         mem_desc: MemDesc::IsPointerTo(Box::new(T::TYPE.info().mem_desc)),
    //     }
    // }

    // types that can only be referenced through pointer
    pub const fn new_opaque<T: 'static>(debug_name: &'static str) -> Self {
        Self {
            unique_id: Some(Either::Left(std::any::TypeId::of::<T>())),
            debug_name,
            layout: Some(Layout::new::<T>()),
            is_zeroable: false,
            drop_fn: Some(drop_impl::<T>),
        }
    }
}

pub type Type = &'static TypeInfo;

unsafe impl Reflect for () {
    const TYPE_INFO: TypeInfo = TypeInfo::new_copy::<()>("()", true);
}

unsafe impl Reflect for u32 {
    const TYPE_INFO: TypeInfo = TypeInfo::new_copy::<u32>("u32", true);
}

unsafe impl Reflect for f64 {
    const TYPE_INFO: TypeInfo = TypeInfo::new_copy::<f64>("f64", true);
}
