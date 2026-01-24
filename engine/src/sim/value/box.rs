use std::ops::Deref;

use crate::util::reflection::{TypeInfo, TypeInfoOptions};

pub struct NlBox<T>(Box<T>);

pub const fn generate_box_type_info<T: 'static>(debug_name: &'static str) -> TypeInfo {
    // it is important that the generated TypeInfo applies to every
    // possible type T
    TypeInfo::new::<NlBox<T>>(TypeInfoOptions {
        debug_name,
        is_zeroable: false,
        mem_repr: Some(&[(0, lir::ValType::Ptr)]),
    })
}

impl<T> Deref for NlBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
