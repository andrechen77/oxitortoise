use std::{
    any::TypeId,
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

use derive_more::derive::{Deref, DerefMut};

use crate::util::reflection::{ConcreteTy, MemRepr, Reflect, TypeInfo, TypeInfoOptions};

#[derive(Deref, DerefMut)]
#[repr(transparent)]
pub struct NlBox<T: Sized>(Box<T>);

impl<T> NlBox<T> {
    pub fn new(value: T) -> Self {
        Self(Box::new(value))
    }
}

unsafe impl<T: Reflect + 'static> Reflect for NlBox<T> {
    fn ty() -> ConcreteTy {
        static TY_BY_TYPE_ID: LazyLock<RwLock<HashMap<TypeId, ConcreteTy>>> =
            LazyLock::new(|| RwLock::new(HashMap::new()));
        if let Some(ty) = TY_BY_TYPE_ID.read().unwrap().get(&TypeId::of::<T>()) {
            ty.clone()
        } else {
            let ty = ConcreteTy::new(&TypeInfo::new_drop::<NlBox<T>>(TypeInfoOptions {
                is_zeroable: false,
                mem_repr: Some(MemRepr::Single(lir::MemOpType::Ptr)),
            }));

            TY_BY_TYPE_ID.write().unwrap().insert(TypeId::of::<T>(), ty.clone());

            ty
        }
    }
}

pub fn generate_box_type_info<T: 'static>() -> TypeInfo {
    // it is important that the generated TypeInfo applies to every
    // possible type T
    TypeInfo::new_drop::<NlBox<T>>(TypeInfoOptions {
        is_zeroable: false,
        mem_repr: Some(MemRepr::Single(lir::ValType::Ptr)),
    })
}
