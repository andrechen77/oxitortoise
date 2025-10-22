use std::any::TypeId;

use lir::smallvec::{SmallVec, smallvec};

/// A concrete type representation in the NetLogo engine. The same NetLogo
/// language type may have multiple concrete type representation.
#[derive(Debug, PartialEq, Clone)]
pub struct NetlogoMachineType(u8);

impl NetlogoMachineType {
    pub const FLOAT: Self = Self(0);
    pub const INTEGER: Self = Self(1);
    pub const STRING: Self = Self(2);
    pub const BOOLEAN: Self = Self(3);
    pub const TURTLE_ID: Self = Self(5);
    pub const PATCH_ID: Self = Self(6);
    pub const POINT: Self = Self(7);
    pub const HEADING: Self = Self(8);
    pub const COLOR: Self = Self(9);
    pub const UNTYPED_PTR: Self = Self(10);
    pub const AGENT_INDEX: Self = Self(11);
    /// A closure that can be passed to `ask`, `create-turtles`, etc.
    pub const ASK_CLOSURE: Self = Self(12);
    pub const DYN_BOX: Self = Self(13);

    /// Returns whether this type is valid and represents the numeric value 0.0
    /// at the all-zero bit pattern, i.e. whether the all-zero bit pattern
    /// is valid as the default value for a NetLogo variable of this type.
    pub fn is_numeric_zeroable(&self) -> bool {
        matches!(self, &Self::FLOAT | &Self::INTEGER | &Self::POINT | &Self::COLOR)
    }

    pub fn to_lir_type(&self) -> SmallVec<[lir::ValType; 1]> {
        match *self {
            Self::FLOAT => smallvec![lir::ValType::F64],
            Self::INTEGER => smallvec![lir::ValType::I64],
            Self::BOOLEAN => smallvec![lir::ValType::I8],
            Self::TURTLE_ID => smallvec![lir::ValType::I64],
            Self::PATCH_ID => smallvec![lir::ValType::I32],
            Self::POINT => smallvec![lir::ValType::F64, lir::ValType::F64],
            Self::HEADING => smallvec![lir::ValType::F64],
            Self::UNTYPED_PTR => smallvec![lir::ValType::Ptr],
            _ => todo!(),
        }
    }
}

impl From<&NetlogoMachineType> for TypeId {
    fn from(value: &NetlogoMachineType) -> Self {
        use crate::sim::{
            color::Color,
            patch::PatchId,
            topology::{Heading, Point},
            turtle::TurtleId,
            value::{Boolean, Float},
        };

        match *value {
            NetlogoMachineType::FLOAT => TypeId::of::<Float>(),
            NetlogoMachineType::INTEGER => TypeId::of::<i32>(),
            NetlogoMachineType::STRING => TypeId::of::<String>(),
            NetlogoMachineType::BOOLEAN => TypeId::of::<Boolean>(),
            NetlogoMachineType::TURTLE_ID => TypeId::of::<TurtleId>(),
            NetlogoMachineType::PATCH_ID => TypeId::of::<PatchId>(),
            NetlogoMachineType::POINT => TypeId::of::<Point>(),
            NetlogoMachineType::HEADING => TypeId::of::<Heading>(),
            NetlogoMachineType::COLOR => TypeId::of::<Color>(),
            _ => panic!("Unknown NetLogo internal type: {:?}", value),
        }
    }
}
