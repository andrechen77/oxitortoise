use derive_more::derive::{From, Not};

use crate::util::reflection::{ConcreteTy, ConstTypeName, Reflect, TypeInfo, TypeInfoOptions};

#[derive(Debug, Clone, Copy, From, PartialEq, Eq, PartialOrd, Ord, Not)]
#[repr(transparent)]
pub struct NlBool(pub bool);

static NL_BOOL_TYPE_INFO: TypeInfo = TypeInfo::new::<NlBool>(TypeInfoOptions {
    is_zeroable: true,
    mem_repr: Some(&[(0, lir::MemOpType::I8)]),
});

unsafe impl Reflect for NlBool {
    const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&NL_BOOL_TYPE_INFO);
}

impl ConstTypeName for NlBool {
    const TYPE_NAME: &'static str = "NlBool";
}
