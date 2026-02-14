use crate::{
    sim::value::{NlBox, r#box::generate_box_type_info},
    util::reflection::{ConcreteTy, ConstTypeName, Reflect, TypeInfo},
};

#[derive(Default, Debug)]
#[allow(dead_code)] // strings will be used eventually, just not at this stage of development
pub struct NlString(String);

impl NlString {
    pub fn new(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl ConstTypeName for NlString {
    const TYPE_NAME: &'static str = "NlString";
}

static STRING_TYPE_INFO: TypeInfo = TypeInfo::new_opaque::<NlString>();
unsafe impl Reflect for NlString {
    const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&STRING_TYPE_INFO);
}

static BOX_STRING_TYPE_INFO: TypeInfo = generate_box_type_info::<NlString>();
unsafe impl Reflect for NlBox<NlString> {
    const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&BOX_STRING_TYPE_INFO);
}
