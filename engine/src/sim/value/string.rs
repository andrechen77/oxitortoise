use macro_reflect::{MirReflect, reflect};

#[derive(Default, Debug, Clone, MirReflect)]
#[allow(dead_code)] // strings will be used eventually, just not at this stage of development
pub struct NlString(String);

impl NlString {
    pub fn new() -> Self {
        Self(String::new())
    }

    pub fn from_str(value: &str) -> Self {
        Self(value.to_string())
    }
}

#[reflect(clone(dynamic))]
impl Reflect for NlString {}

#[reflect]
impl Reflect for &NlString {}
