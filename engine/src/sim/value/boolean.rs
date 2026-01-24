use derive_more::derive::{From, Not};

use crate::util::reflection::{ConcreteTy, Reflect, TypeInfo, TypeInfoOptions};

#[derive(Debug, Clone, Copy, From, PartialEq, Eq, PartialOrd, Ord, Not)]
#[repr(transparent)]
pub struct NlBool(pub bool);

static NL_BOOL_TYPE_INFO: TypeInfo = TypeInfo::new::<NlBool>(TypeInfoOptions {
    debug_name: "NlBool",
    is_zeroable: true,
    mem_repr: Some(&[(0, lir::ValType::I8)]),
});

impl Reflect for NlBool {
    const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&NL_BOOL_TYPE_INFO);
}
