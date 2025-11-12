use derive_more::derive::{From, Not};

use crate::util::type_registry::{Reflect, TypeInfo, TypeInfoOptions};

#[derive(Debug, Clone, Copy, From, PartialEq, Eq, PartialOrd, Ord, Not)]
#[repr(transparent)]
pub struct NlBool(pub bool);

static NL_BOOL_TYPE_INFO: TypeInfo = TypeInfo::new::<NlBool>(TypeInfoOptions {
    debug_name: "NlBool",
    is_zeroable: true,
    lir_repr: Some(&[lir::ValType::I8]),
});

impl Reflect for NlBool {
    const TYPE_INFO: &TypeInfo = &NL_BOOL_TYPE_INFO;
}
