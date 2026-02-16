use std::alloc::Layout;

// QUESTION what is this the correct place to put the unsafe invariant?
// the boundary between safe and unsafe code is hard to think about when the
// entire compiler is one big unsafe abstraction. however, if possible I'd like
// to establish a boundary where authors are expected to double-triple check
// that their associated `TypeInfo` is correct.

/// A trait to indicate that the compiler can generate code to manipulate values
/// of this type.
///
/// # Safety
///
/// Implementors must guarantee that the associated `TypeInfo` is correct, as
/// the information will be used to generate and run unsafe code.
pub unsafe trait Reflect: 'static {
    /// N.B. This should be defined as a reference to an actual static object,
    /// as in the following.
    /// ```rust
    /// use oxitortoise_engine::util::reflection::{ConcreteTy, Reflect, TypeInfo, TypeInfoOptions};
    ///
    /// struct MyType;
    /// static MY_TYPE_INFO: TypeInfo = TypeInfo::new::<MyType>(TypeInfoOptions {
    ///     debug_name: "MyType",
    ///     is_zeroable: true,
    ///     lir_repr: None,
    /// });
    /// impl Reflect for MyType {
    ///     const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&MY_TYPE_INFO);
    /// }
    /// ```
    /// DO NOT define it like the following.
    /// ```rust
    /// use oxitortoise_engine::util::reflection::{ConcreteTy, Reflect, TypeInfo, TypeInfoOptions};
    ///
    /// struct MyType;
    /// impl Reflect for MyType {
    ///     const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&TypeInfo::new::<MyType>(TypeInfoOptions {
    ///        debug_name: "MyType",
    ///         is_zeroable: true,
    ///         lir_repr: None,
    ///     }));
    /// }
    /// ```
    /// Defining it like the above may create references to different objects
    /// each time the constant is used; these objects will not compare equal.
    const CONCRETE_TY: ConcreteTy;
}

/// Information about a type that is used by the engine to generate code that
/// manipulates values of the corresponding type.
#[derive(Debug, Clone, Copy)]
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
    /// Each value that maps as a separate register when this type is
    /// loaded from or stored into memory corresponds to a tuple of the offset
    /// and LIR type of the value.
    pub mem_repr: Option<&'static [(usize, lir::MemOpType)]>,
}

/// A helper struct to pass as options to [`TypeInfo::new`]
pub struct TypeInfoOptions {
    pub is_zeroable: bool,
    pub mem_repr: Option<&'static [(usize, lir::MemOpType)]>,
}

unsafe fn drop_impl<T>(ptr: *mut u8) {
    unsafe {
        std::ptr::drop_in_place(ptr as *mut T);
    }
}

impl TypeInfo {
    /// Generates a `TypeInfo` for the given type, where all fields are
    /// guaranteed correct except for those specified in the `options`
    /// parameter.
    pub const fn new_drop<T: 'static + ConstTypeName>(options: TypeInfoOptions) -> Self {
        let TypeInfoOptions { is_zeroable, mem_repr } = options;

        Self {
            debug_name: T::TYPE_NAME,
            layout: Layout::new::<T>(),
            is_zeroable,
            drop_fn: Some(drop_impl::<T>),
            mem_repr,
        }
    }

    pub const fn new_copy<T: 'static + Copy + ConstTypeName>(options: TypeInfoOptions) -> Self {
        let TypeInfoOptions { is_zeroable, mem_repr } = options;

        Self {
            debug_name: T::TYPE_NAME,
            layout: Layout::new::<T>(),
            is_zeroable,
            drop_fn: None,
            mem_repr,
        }
    }

    // types that can only be referenced through pointer
    pub const fn new_opaque<T: 'static + ConstTypeName>() -> Self {
        Self {
            debug_name: T::TYPE_NAME,
            layout: Layout::new::<T>(),
            is_zeroable: false,
            drop_fn: Some(drop_impl::<T>),
            mem_repr: None,
        }
    }
}

/// A concrete type representation in the NetLogo engine. The same NetLogo
/// language type may have multiple concrete type representation.
#[derive(Clone, Copy, derive_more::Debug)]
#[debug("{}", self.info().debug_name)]
pub struct ConcreteTy(&'static TypeInfo);

impl ConcreteTy {
    pub const fn new(info: &'static TypeInfo) -> Self {
        Self(info)
    }

    pub const fn info(&self) -> &'static TypeInfo {
        self.0
    }
}

impl PartialEq for ConcreteTy {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0 as *const TypeInfo, other.0 as *const TypeInfo)
    }
}

// TODO(wishlist) once stable, use const_type_name to automatically generate this
/// A trait to fill in for the `const_type_name` until it is stable.
pub trait ConstTypeName {
    const TYPE_NAME: &'static str;
}
