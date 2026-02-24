use std::sync::LazyLock;

use crate::util::reflection::{ConcreteTy, Reflect, TypeInfo};

#[derive(Default, Debug)]
#[allow(dead_code)] // strings will be used eventually, just not at this stage of development
pub struct NlString(String);

impl NlString {
    pub fn new(value: &str) -> Self {
        Self(value.to_string())
    }
}

unsafe impl Reflect for NlString {
    fn ty() -> ConcreteTy {
        static TY: LazyLock<ConcreteTy> =
            LazyLock::new(|| ConcreteTy::new(&TypeInfo::new_opaque::<NlString>()));
        TY.clone()
    }
}
