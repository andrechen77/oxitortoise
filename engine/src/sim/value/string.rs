use macro_reflect::{ReflectComponents, reflect};

#[derive(Default, Debug, Clone, ReflectComponents)]
#[allow(dead_code)] // strings will be used eventually, just not at this stage of development
pub struct NlString(String);

impl NlString {
    pub fn new(value: &str) -> Self {
        Self(value.to_string())
    }
}

#[reflect(clone(dynamic))]
impl Reflect for NlString {}

#[reflect]
impl Reflect for &NlString {}
