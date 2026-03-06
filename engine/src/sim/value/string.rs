use crate::mir::reflection::{Reflect, Type, TypeInfo};

#[derive(Default, Debug)]
#[allow(dead_code)] // strings will be used eventually, just not at this stage of development
pub struct NlString(String);

impl NlString {
    pub fn new(value: &str) -> Self {
        Self(value.to_string())
    }
}

unsafe impl Reflect for NlString {
    const TYPE: Type = Type::new(&TypeInfo::new_opaque::<NlString>("NlString"));
}
