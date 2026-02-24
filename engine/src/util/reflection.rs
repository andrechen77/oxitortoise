use std::{
    alloc::Layout,
    sync::{Arc, LazyLock},
};

/// A trait to indicate that the compiler can generate code to manipulate values
/// of this type.
///
/// # Safety
///
/// Implementors must guarantee that the associated `TypeInfo` is correct, as
/// the information will be used to generate and run unsafe code.
pub unsafe trait Reflect: 'static {
    // TODO update the docs to reflect the new API
    /// This should return the same `ConcreteTy` instance every time it is called. See [`ConcreteTy::new`].
    /// ```rust
    /// use oxitortoise_engine::util::reflection::{ConcreteTy, Reflect, TypeInfo, TypeInfoOptions, ConstTypeName};
    ///
    /// struct MyType;
    /// unsafe impl Reflect for MyType {
    ///     const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&MY_TYPE_INFO);
    ///     fn ty() -> ConcreteTy {
    ///         static TY: LazyLock<ConcreteTy> = LazyLock::new(|| {
    ///             ConcreteTy::new(&TypeInfo::new_drop::<MyType>(TypeInfoOptions {
    ///                 is_zeroable: true,
    ///                 mem_repr: None,
    ///             }))
    ///         });
    ///         TY.clone()
    ///     }
    /// }
    /// ```
    /// DO NOT define it like the following. This will create new `ConcreteTy`
    /// instances each time it is called which will not compare equal.
    /// ```rust
    /// use oxitortoise_engine::util::reflection::{ConcreteTy, Reflect, TypeInfo, TypeInfoOptions, ConstTypeName};
    ///
    /// struct MyType;
    /// unsafe impl Reflect for MyType {
    ///     const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&MY_TYPE_INFO);
    ///     fn ty() -> ConcreteTy {
    ///         ConcreteTy::new(&TypeInfo::new_drop::<MyType>(TypeInfoOptions {
    ///             is_zeroable: true,
    ///             mem_repr: None,
    ///         }))
    ///     }
    /// }
    /// ```
    /// Defining it like the above may create references to different objects
    /// each time the constant is used; these objects will not compare equal.
    fn ty() -> ConcreteTy;
}

/// Information about a type that is used by the engine to generate code that
/// manipulates values of the corresponding type.
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum MemRepr {
    /// Represents the fields of the type in memory along with their offsets.
    Compound(Vec<(usize, ConcreteTy)>),
    /// Represents the type as a single LIR value.
    Single(lir::ValType),
}

/// A helper struct to pass as options to [`TypeInfo::new`]
pub struct TypeInfoOptions {
    pub is_zeroable: bool,
    pub mem_repr: Option<MemRepr>,
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
    pub fn new_drop<T: 'static>(options: TypeInfoOptions) -> Self {
        let TypeInfoOptions { is_zeroable, mem_repr } = options;

        Self {
            debug_name: std::any::type_name::<T>(),
            layout: Layout::new::<T>(),
            is_zeroable,
            drop_fn: Some(drop_impl::<T>),
            mem_repr,
        }
    }

    pub fn new_copy<T: 'static + Copy>(options: TypeInfoOptions) -> Self {
        let TypeInfoOptions { is_zeroable, mem_repr } = options;

        Self {
            debug_name: std::any::type_name::<T>(),
            layout: Layout::new::<T>(),
            is_zeroable,
            drop_fn: None,
            mem_repr,
        }
    }

    // types that can only be referenced through pointer
    pub fn new_opaque<T: 'static>() -> Self {
        Self {
            debug_name: std::any::type_name::<T>(),
            layout: Layout::new::<T>(),
            is_zeroable: false,
            drop_fn: Some(drop_impl::<T>),
            mem_repr: None,
        }
    }
}

/// A concrete type representation in the NetLogo engine. The same NetLogo
/// language type may have multiple concrete type representation.
#[derive(Clone, derive_more::Debug)]
#[debug("{}", self.info().debug_name)]
pub struct ConcreteTy(Arc<TypeInfo>);

impl ConcreteTy {
    /// If this function is called multiple times with the same `info`, it will
    /// return different `ConcreteTy` instances that will not compare equal.
    /// Call this function once per type.
    pub fn new(info: &TypeInfo) -> Self {
        Self(Arc::new(info.clone()))
    }

    pub fn info(&self) -> &TypeInfo {
        &self.0
    }
}

impl PartialEq for ConcreteTy {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(Arc::as_ptr(&self.0), Arc::as_ptr(&other.0))
    }
}

// static UNTYPED_PTR_INFO: TypeInfo = TypeInfo::new_copy::<*mut u8>(TypeInfoOptions {
//     is_zeroable: false,
//     mem_repr: Some(&[(0, lir::MemOpType::Ptr)]),
// });
// unsafe impl Reflect for *mut u8 {
//     const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&UNTYPED_PTR_INFO);
// }

unsafe impl Reflect for () {
    fn ty() -> ConcreteTy {
        static TY: LazyLock<ConcreteTy> = LazyLock::new(|| {
            ConcreteTy::new(&TypeInfo::new_copy::<()>(TypeInfoOptions {
                is_zeroable: true,
                mem_repr: Some(MemRepr::Single(lir::ValType::Ptr)),
            }))
        });
        TY.clone()
    }
}

unsafe impl Reflect for u32 {
    fn ty() -> ConcreteTy {
        static TY: LazyLock<ConcreteTy> = LazyLock::new(|| {
            ConcreteTy::new(&TypeInfo::new_copy::<u32>(TypeInfoOptions {
                is_zeroable: false,
                mem_repr: Some(MemRepr::Single(lir::ValType::I32)),
            }))
        });
        TY.clone()
    }
}

unsafe impl Reflect for f64 {
    fn ty() -> ConcreteTy {
        static TY: LazyLock<ConcreteTy> = LazyLock::new(|| {
            ConcreteTy::new(&TypeInfo::new_copy::<f32>(TypeInfoOptions {
                is_zeroable: true,
                mem_repr: Some(MemRepr::Single(lir::ValType::F64)),
            }))
        });
        TY.clone()
    }
}
