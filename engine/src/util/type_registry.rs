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
    /// use oxitortoise_engine::util::type_registry::{Reflect, TypeInfo, TypeInfoOptions};
    ///
    /// struct MyType;
    /// static MY_TYPE_INFO: TypeInfo = TypeInfo::new::<MyType>(TypeInfoOptions {
    ///     debug_name: "MyType",
    ///     is_zeroable: true,
    ///     lir_repr: None,
    /// });
    /// impl Reflect for MyType {
    ///     const TYPE_INFO: &TypeInfo = &MY_TYPE_INFO;
    /// }
    /// ```
    /// DO NOT define it like the following.
    /// ```rust
    /// use oxitortoise_engine::util::type_registry::{Reflect, TypeInfo, TypeInfoOptions};
    ///
    /// struct MyType;
    /// impl Reflect for MyType {
    ///     const TYPE_INFO: &TypeInfo = &TypeInfo::new::<MyType>(TypeInfoOptions {
    ///        debug_name: "MyType",
    ///         is_zeroable: true,
    ///         lir_repr: None,
    ///     });
    /// }
    /// ```
    /// Defining it like the above may create references to different objects
    /// each time the constant is used; these objects will not compare equal.
    const TYPE_INFO: &TypeInfo;
}

/// Information about a type that is used by the engine to generate code that
/// manipulates values of the corresponding type.
#[derive(Debug)]
pub struct TypeInfo {
    pub debug_name: &'static str,
    // pub type_id: TypeId,
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
            // type_id: TypeId::of::<T>(),
            layout: Layout::new::<T>(),
            is_zeroable,
            drop_fn: drop_impl::<T>,
            lir_repr,
        }
    }
}

// static REGISTRY: RwLock<Option<HashMap<TypeId, &'static TypeInfo>>> = RwLock::new(None);

// pub fn register_type_info<T: Reflect>() {
//     let type_info: &'static TypeInfo = Box::leak(Box::new(TypeInfo::new::<T>()));

//     let mut reg_guard = REGISTRY.write().unwrap();
//     reg_guard.get_or_insert_default().insert(TypeId::of::<T>(), type_info);
// }

// pub unsafe fn register_type_info_unchecked(type_info: &'static TypeInfo) {
//     let mut reg_guard = REGISTRY.write().unwrap();
//     reg_guard.get_or_insert_default().insert()
// }

// pub fn get_type_info(r#type: TypeId) -> &'static TypeInfo {
//     let reg_guard = REGISTRY.read().unwrap(); // TODO handle error
//     reg_guard
//         .as_ref()
//         .and_then(|reg| reg.get(&r#type))
//         .unwrap_or_else(|| panic!("type info not found in registry for type {:?}", r#type))
// }

// fn init_type_info_registry() -> HashMap<TypeId, Arc<TypeInfo>> {
//     let mut m = HashMap::new();
//     macro_rules! reg {
//         ($t:ty, $is_zeroable:expr) => {
//             m.insert(TypeId::of::<$t>(), Arc::new(generate_type_info::<$t>($is_zeroable)));
//         };
//     }
//     reg!(u8, true);
//     reg!(u16, true);
//     reg!(u32, true);
//     reg!(u64, true);
//     reg!(i8, true);
//     reg!(i16, true);
//     reg!(i32, true);
//     reg!(i64, true);
//     reg!(f32, true);
//     reg!(f64, true);
//     reg!(bool, true);
//     reg!(String, false);
//     reg!(crate::sim::value::Float, true);
//     reg!(crate::sim::topology::Heading, true);
//     reg!(crate::sim::topology::Point, true);
//     reg!(crate::sim::color::Color, true);
//     reg!(crate::sim::turtle::TurtleId, true);
//     reg!(crate::sim::patch::PatchId, true);
//     reg!(crate::sim::value::Boolean, true);
//     reg!(crate::sim::turtle::TurtleBaseData, false);
//     reg!(crate::sim::patch::PatchBaseData, false);

//     // println!("type registry: {:?}", m);

//     m
// }
