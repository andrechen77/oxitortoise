use std::{alloc::Layout, sync::Arc};

// TODO what to do about lifetimes? could cause unsafety and sadness

/// A trait to indicate that the compiler can generate code to manipulate values
/// of this type.
///
/// # Safety
///
/// Implementors must guarantee that the associated `TypeInfo` is correct, as
/// the information will be used to generate and run unsafe code.
pub unsafe trait Reflect {
    const TYPE_INFO: TypeInfo;
}

/// Information about a type that is used by the engine to generate code that
/// manipulates values of the corresponding type.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeInfo {
    pub debug_name: &'static str,
    pub layout: Layout,
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
    /// The memory representation of the type. None if this is an opaque type.
    pub mem_repr: Option<MemRepr>,
}

// potential optimization: instead of having the Pointer variant reference
// the ConcreteTy directly, we could have list the fields of the pointee so that
// we don't need separate TypeInfo for both T and &mut T if all we want is to
// use &mut T. This is specifically for &mut Workspace, since we never actually
// need to drop it, yet we still need to implement Workspace: Reflect to get it
// it to.
// The 'const' refers to the fact that the type can be used in a const definition.
#[derive(Debug, Clone, PartialEq)]
pub enum MemRepr {
    /// The type consists of the following fields as its immediate children,
    /// with associated byte offsets. This does not need to be a comprehensive
    /// list, just the fields that the type wants to expose for foreign code
    /// to access.
    Compound(&'static [(usize, &'static TypeInfo)]),
    /// The type is a pointer to another type.
    Pointer { pointee_ty: &'static TypeInfo },
    /// The type is an array of another type.
    Array { element_ty: &'static TypeInfo, length: usize },
    /// The type is a single LIR value.
    Single(lir::ValType),
}

unsafe fn drop_impl<T>(ptr: *mut u8) {
    unsafe {
        std::ptr::drop_in_place(ptr as *mut T);
    }
}

impl TypeInfo {
    pub const fn new_drop<T: 'static>(debug_name: &'static str, mem_repr: MemRepr) -> Self {
        Self {
            debug_name,
            layout: Layout::new::<T>(),
            is_zeroable: false,
            drop_fn: Some(drop_impl::<T>),
            mem_repr: Some(mem_repr),
        }
    }

    pub const fn new_drop_zeroable<T: 'static>(
        debug_name: &'static str,
        mem_repr: MemRepr,
    ) -> Self {
        Self {
            debug_name,
            layout: Layout::new::<T>(),
            is_zeroable: true,
            drop_fn: Some(drop_impl::<T>),
            mem_repr: Some(mem_repr),
        }
    }

    pub const fn new_copy<T: 'static + Copy>(
        debug_name: &'static str,
        is_zeroable: bool,
        mem_repr: MemRepr,
    ) -> Self {
        Self {
            debug_name,
            layout: Layout::new::<T>(),
            is_zeroable,
            drop_fn: None,
            mem_repr: Some(mem_repr),
        }
    }

    pub const fn new_mut_ref_to<T: Reflect + 'static>(debug_name: &'static str) -> Self {
        Self {
            debug_name,
            layout: Layout::new::<&mut T>(),
            is_zeroable: false,
            drop_fn: None, // mut refs have no destructor, so this is correct
            mem_repr: Some(MemRepr::Pointer { pointee_ty: &T::TYPE_INFO }),
        }
    }

    // types that can only be referenced through pointer
    pub const fn new_opaque<T: 'static>(debug_name: &'static str) -> Self {
        Self {
            debug_name,
            layout: Layout::new::<T>(),
            is_zeroable: false,
            drop_fn: Some(drop_impl::<T>),
            mem_repr: None,
        }
    }
}

/// A concrete type representation in the NetLogo engine. The same NetLogo
/// language type may have multiple concrete type representations.
#[derive(Clone, derive_more::Debug)]
pub enum ConcreteTy {
    #[debug("{}", self.info().debug_name)]
    Dynamic(Arc<TypeInfo>),
    #[debug("{}", self.info().debug_name)]
    Static(&'static TypeInfo),
}

impl ConcreteTy {
    pub fn info(&self) -> &TypeInfo {
        match self {
            ConcreteTy::Dynamic(info) => info,
            ConcreteTy::Static(info) => info,
        }
    }
}

impl PartialEq for ConcreteTy {
    fn eq(&self, other: &Self) -> bool {
        self.info() == other.info()
    }
}

impl PartialEq<TypeInfo> for ConcreteTy {
    fn eq(&self, other: &TypeInfo) -> bool {
        self.info() == other
    }
}

impl From<&ConcreteTy> for ConcreteTy {
    fn from(ty: &ConcreteTy) -> Self {
        ty.clone()
    }
}

impl From<&'static TypeInfo> for ConcreteTy {
    fn from(info: &'static TypeInfo) -> Self {
        ConcreteTy::Static(info)
    }
}

impl From<&&'static TypeInfo> for ConcreteTy {
    fn from(info: &&'static TypeInfo) -> Self {
        ConcreteTy::Static(info)
    }
}

unsafe impl Reflect for () {
    const TYPE_INFO: TypeInfo =
        TypeInfo::new_copy::<()>("()", true, MemRepr::Single(lir::ValType::Ptr));
}

unsafe impl Reflect for u32 {
    const TYPE_INFO: TypeInfo =
        TypeInfo::new_copy::<u32>("u32", false, MemRepr::Single(lir::ValType::I32));
}

unsafe impl Reflect for f64 {
    const TYPE_INFO: TypeInfo =
        TypeInfo::new_copy::<f64>("f64", true, MemRepr::Single(lir::ValType::F64));
}
