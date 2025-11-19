//! NetLogo runtime values.

pub mod agentset;
mod boolean;
mod r#box;
mod dynbox;
mod float;
mod list;

pub use boolean::NlBool;
pub use r#box::NlBox;
pub use dynbox::DynBox;
pub use dynbox::UnpackedDynBox;
pub use float::NlFloat;
pub use list::NlList;

use crate::util::reflection::ConcreteTy;
use crate::util::reflection::Reflect;
use crate::util::reflection::TypeInfo;
use crate::util::reflection::TypeInfoOptions;

static UNTYPED_PTR_INFO: TypeInfo = TypeInfo::new::<*mut u8>(TypeInfoOptions {
    debug_name: "UntypedPtr",
    is_zeroable: false,
    lir_repr: Some(&[lir::ValType::Ptr]),
});
impl Reflect for *mut u8 {
    const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&UNTYPED_PTR_INFO);
}

static UNIT_INFO: TypeInfo = TypeInfo::new::<()>(TypeInfoOptions {
    debug_name: "Unit",
    is_zeroable: false,
    lir_repr: Some(&[]),
});
impl Reflect for () {
    const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&UNIT_INFO);
}

// QUESTION should this type even exist? if so, we'd need a unified place to
// put all information about it such as the fact that it's 32 bits.
static AGENT_INDEX_INFO: TypeInfo = TypeInfo::new::<u32>(TypeInfoOptions {
    debug_name: "AgentIndex",
    is_zeroable: false,
    lir_repr: Some(&[lir::ValType::I32]),
});
pub const AGENT_INDEX_CONCRETE_TY: ConcreteTy = ConcreteTy::new(&AGENT_INDEX_INFO);

// TODO(mvp) add box-like representation for indirect values.
// Values such as strings, lists, anonymous procedures, etc. should use this,
// replacing the existing String file
