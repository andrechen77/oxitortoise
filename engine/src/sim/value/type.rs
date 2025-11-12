use crate::{
    sim::{
        color::Color,
        patch::PatchId,
        topology::{Heading, Point},
        turtle::TurtleId,
        value::{DynBox, NlBool, NlFloat, NlString},
    },
    util::type_registry::{Reflect as _, TypeInfo, TypeInfoOptions},
};

/// A concrete type representation in the NetLogo engine. The same NetLogo
/// language type may have multiple concrete type representation.
#[derive(Debug, Clone, Copy)]
pub struct NlMachineTy(&'static TypeInfo);

impl NlMachineTy {
    pub fn new(info: &'static TypeInfo) -> Self {
        Self(info)
    }
}

impl PartialEq for NlMachineTy {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0 as *const TypeInfo, other.0 as *const TypeInfo)
    }
}

const UNTYPED_PTR_INFO: &TypeInfo = &TypeInfo::new::<*mut u8>(TypeInfoOptions {
    debug_name: "UntypedPtr",
    is_zeroable: false,
    lir_repr: Some(&[lir::ValType::Ptr]),
});

const UNIT_INFO: &TypeInfo = &TypeInfo::new::<()>(TypeInfoOptions {
    debug_name: "Unit",
    is_zeroable: false,
    lir_repr: Some(&[]),
});

// QUESTION should this type even exist? if so, we'd need a unified place to
// put all information about it such as the fact that it's 32 bits.
const AGENT_INDEX_INFO: &TypeInfo = &TypeInfo::new::<u32>(TypeInfoOptions {
    debug_name: "AgentIndex",
    is_zeroable: false,
    lir_repr: Some(&[lir::ValType::I32]),
});

impl NlMachineTy {
    pub const FLOAT: Self = Self(NlFloat::TYPE_INFO);
    pub const STRING: Self = Self(NlString::TYPE_INFO);
    pub const BOOLEAN: Self = Self(NlBool::TYPE_INFO);
    pub const TURTLE_ID: Self = Self(TurtleId::TYPE_INFO);
    pub const PATCH_ID: Self = Self(PatchId::TYPE_INFO);
    pub const POINT: Self = Self(Point::TYPE_INFO);
    pub const HEADING: Self = Self(Heading::TYPE_INFO);
    pub const COLOR: Self = Self(Color::TYPE_INFO);
    pub const UNTYPED_PTR: Self = Self(UNTYPED_PTR_INFO);
    pub const AGENT_INDEX: Self = Self(AGENT_INDEX_INFO);
    pub const DYN_BOX: Self = Self(DynBox::TYPE_INFO);
    pub const UNIT: Self = Self(UNIT_INFO);

    pub fn info(&self) -> &'static TypeInfo {
        self.0
    }
}
