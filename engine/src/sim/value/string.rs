use crate::util::type_registry::{Reflect, TypeInfo, TypeInfoOptions};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NlString(Box<()>);
// for now, any non-copy pointer-sized type will do, to simulate what the
// actual string type would look like

impl NlString {
    pub fn new() -> Self {
        Self(Box::new(()))
    }
}

impl From<&str> for NlString {
    fn from(_value: &str) -> Self {
        todo!()
    }
}

impl Default for NlString {
    fn default() -> Self {
        Self::new()
    }
}

static NL_STRING_TYPE_INFO: TypeInfo = TypeInfo::new::<NlString>(TypeInfoOptions {
    debug_name: "NlString",
    is_zeroable: false,
    lir_repr: Some(&[lir::ValType::Ptr]),
});

impl Reflect for NlString {
    const TYPE_INFO: &TypeInfo = &NL_STRING_TYPE_INFO;
}
