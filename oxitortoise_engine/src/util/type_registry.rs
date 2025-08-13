use std::{
    alloc::Layout,
    any::TypeId,
    collections::HashMap,
    sync::{Arc, OnceLock, RwLock},
};

#[derive(Debug)]
pub struct TypeInfo {
    pub human_name: &'static str,
    pub layout: Layout,
    /// Whether this type is valid at the all-zero bit pattern.
    pub is_zeroable: bool,
    pub drop_fn: unsafe fn(*mut ()),
}

pub const fn generate_type_info<T: 'static>(is_zeroable: bool) -> TypeInfo {
    unsafe fn drop_impl<T>(ptr: *mut ()) {
        // SAFETY: caller guarantees that ptr is a valid pointer to T that can
        // be dropped
        unsafe {
            std::ptr::drop_in_place(ptr as *mut T);
        }
    }
    TypeInfo {
        human_name: std::any::type_name::<T>(),
        layout: Layout::new::<T>(),
        is_zeroable,
        drop_fn: drop_impl::<T>,
    }
}

fn init_type_info_registry() -> HashMap<TypeId, Arc<TypeInfo>> {
    let mut m = HashMap::new();
    macro_rules! reg {
        ($t:ty, $is_zeroable:expr) => {
            m.insert(TypeId::of::<$t>(), Arc::new(generate_type_info::<$t>($is_zeroable)));
        };
    }
    reg!(u8, true);
    reg!(u16, true);
    reg!(u32, true);
    reg!(u64, true);
    reg!(i8, true);
    reg!(i16, true);
    reg!(i32, true);
    reg!(i64, true);
    reg!(f32, true);
    reg!(f64, true);
    reg!(bool, true);
    reg!(String, false);
    reg!(crate::sim::value::Float, true);
    reg!(crate::sim::topology::Heading, true);
    reg!(crate::sim::topology::Point, true);
    reg!(crate::sim::color::Color, true);
    reg!(crate::sim::turtle::TurtleId, true);
    reg!(crate::sim::patch::PatchId, true);
    reg!(crate::sim::value::Boolean, true);
    reg!(crate::sim::turtle::TurtleBaseData, false);
    reg!(crate::sim::patch::PatchBaseData, false);

    // println!("type registry: {:?}", m);

    m
}

static TYPEINFO_REGISTRY: OnceLock<RwLock<HashMap<TypeId, Arc<TypeInfo>>>> = OnceLock::new();

fn get_type_info_registry() -> &'static RwLock<HashMap<TypeId, Arc<TypeInfo>>> {
    TYPEINFO_REGISTRY.get_or_init(|| RwLock::new(init_type_info_registry()))
}

pub fn get_type_info(r#type: TypeId) -> Arc<TypeInfo> {
    let registry = get_type_info_registry().read().unwrap();
    registry
        .get(&r#type)
        .unwrap_or_else(|| panic!("TypeInfo for {:?} not found in registry", r#type))
        .clone()
}

pub fn register_type_info<T: 'static>(is_zeroable: bool) {
    let type_info = generate_type_info::<T>(is_zeroable);
    get_type_info_registry().write().unwrap().insert(TypeId::of::<T>(), Arc::new(type_info));
}
