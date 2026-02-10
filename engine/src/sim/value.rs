//! NetLogo runtime values.

pub mod agentset;
mod any;
mod boolean;
mod r#box;
mod float;
mod list;

pub use any::{BoxedAny, PackedAny, UnpackedAny};
pub use boolean::NlBool;
pub use r#box::NlBox;
pub use float::NlFloat;
pub use list::NlList;

use crate::util::reflection::ConcreteTy;
use crate::util::reflection::ConstTypeName;
use crate::util::reflection::Reflect;
use crate::util::reflection::TypeInfo;
use crate::util::reflection::TypeInfoOptions;

static UNTYPED_PTR_INFO: TypeInfo = TypeInfo::new::<*mut u8>(TypeInfoOptions {
    is_zeroable: false,
    mem_repr: Some(&[(0, lir::MemOpType::Ptr)]),
});
unsafe impl Reflect for *mut u8 {
    const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&UNTYPED_PTR_INFO);
}

impl ConstTypeName for *mut u8 {
    const TYPE_NAME: &'static str = "*mut u8";
}

static UNIT_INFO: TypeInfo =
    TypeInfo::new::<()>(TypeInfoOptions { is_zeroable: false, mem_repr: Some(&[]) });
unsafe impl Reflect for () {
    const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&UNIT_INFO);
}

impl ConstTypeName for () {
    const TYPE_NAME: &'static str = "()";
}

// QUESTION should this type even exist? if so, we'd need a unified place to
// put all information about it such as the fact that it's 32 bits.
static U32_INFO: TypeInfo = TypeInfo::new::<u32>(TypeInfoOptions {
    is_zeroable: false,
    mem_repr: Some(&[(0, lir::MemOpType::I32)]),
});
pub const U32_CONCRETE_TY: ConcreteTy = ConcreteTy::new(&U32_INFO);

impl ConstTypeName for u32 {
    const TYPE_NAME: &'static str = "u32";
}

// TODO(mvp) add box-like representation for indirect values.
// Values such as strings, lists, anonymous procedures, etc. should use this,
// replacing the existing String file
