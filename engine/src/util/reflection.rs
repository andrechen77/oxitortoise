use std::alloc::Layout;

// QUESTION what is this the correct place to put the unsafe invariant?
// the boundary between safe and unsafe code is hard to think about when the
// entire compiler is one big unsafe abstraction. however, if possible I'd like
// to establish a boundary where authors are expected to double-triple check
// that their associated `TypeInfo` is correct.

pub trait Reflect: 'static {
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
#[derive(Debug)]
pub struct TypeInfo {
    pub debug_name: &'static str,
    pub layout: Layout,
    /// Whether this type is valid at the all-zero bit pattern *and* represents
    /// the numeric value 0.0.
    pub is_zeroable: bool,
    /// The drop function for this type.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the passed pointer is a valid pointer to
    /// T that can be dropped, and that that value will never be used again.
    pub drop_fn: unsafe fn(*mut ()),
    pub lir_repr: Option<&'static [lir::ValType]>,
}

/// A helper struct to pass as options to [`TypeInfo::new`]
pub struct TypeInfoOptions {
    // TODO(wishlist) once stable, use const_type_name to automatically generate this
    pub debug_name: &'static str,
    pub is_zeroable: bool,
    pub lir_repr: Option<&'static [lir::ValType]>,
}

impl TypeInfo {
    /// Generates a `TypeInfo` for the given type, where all fields are
    /// guaranteed correct except for those specified in the `options`
    /// parameter.
    pub const fn new<T: 'static>(options: TypeInfoOptions) -> Self {
        unsafe fn drop_impl<T>(ptr: *mut ()) {
            // SAFETY: it is part of the precondition of the
            // `Reflection::drop_fn` field that the value is valid and can be
            // dropped
            unsafe {
                std::ptr::drop_in_place(ptr as *mut T);
            }
        }

        let TypeInfoOptions { debug_name, is_zeroable, lir_repr } = options;

        Self {
            debug_name,
            layout: Layout::new::<T>(),
            is_zeroable,
            drop_fn: drop_impl::<T>,
            lir_repr,
        }
    }
}

/// A concrete type representation in the NetLogo engine. The same NetLogo
/// language type may have multiple concrete type representation.
#[derive(Debug, Clone, Copy)]
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
