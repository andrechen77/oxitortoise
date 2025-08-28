use std::any::TypeId;

use lir::smallvec::{SmallVec, smallvec};

/// A concrete type representation in the NetLogo engine. The same NetLogo
/// language type may have multiple concrete type representation.
#[derive(Debug, PartialEq, Clone)]
pub struct NetlogoInternalType(u8);

impl NetlogoInternalType {
    pub const FLOAT: Self = Self(0);
    pub const INTEGER: Self = Self(1);
    pub const STRING: Self = Self(2);
    pub const BOOLEAN: Self = Self(3);
    pub const TURTLE_ID: Self = Self(5);
    pub const PATCH_ID: Self = Self(6);
    pub const POINT: Self = Self(7);
    pub const HEADING: Self = Self(8);
    pub const COLOR: Self = Self(9);

    /// Returns whether this type is valid and represents the numeric value 0.0
    /// at the all-zero bit pattern, i.e. whether the all-zero bit pattern
    /// is valid as the default value for a NetLogo variable of this type.
    pub fn is_numeric_zeroable(&self) -> bool {
        match self {
            &Self::FLOAT | &Self::INTEGER | &Self::POINT | &Self::COLOR => true,
            _ => false,
        }
    }

    pub fn to_lir_type(&self) -> SmallVec<[lir::ValType; 1]> {
        match self {
            &Self::FLOAT => smallvec![lir::ValType::F64],
            &Self::INTEGER => smallvec![lir::ValType::I64],
            &Self::BOOLEAN => smallvec![lir::ValType::I8],
            &Self::TURTLE_ID => smallvec![lir::ValType::I64],
            &Self::PATCH_ID => smallvec![lir::ValType::I32],
            &Self::POINT => smallvec![lir::ValType::F64, lir::ValType::F64],
            &Self::HEADING => smallvec![lir::ValType::F64],
            _ => todo!(),
        }
    }
}

impl From<&NetlogoInternalType> for TypeId {
    fn from(value: &NetlogoInternalType) -> Self {
        use crate::sim::{
            color::Color,
            patch::PatchId,
            topology::{Heading, Point},
            turtle::TurtleId,
            value::{Boolean, Float},
        };

        match value {
            &NetlogoInternalType::FLOAT => TypeId::of::<Float>(),
            &NetlogoInternalType::INTEGER => TypeId::of::<i32>(),
            &NetlogoInternalType::STRING => TypeId::of::<String>(),
            &NetlogoInternalType::BOOLEAN => TypeId::of::<Boolean>(),
            &NetlogoInternalType::TURTLE_ID => TypeId::of::<TurtleId>(),
            &NetlogoInternalType::PATCH_ID => TypeId::of::<PatchId>(),
            &NetlogoInternalType::POINT => TypeId::of::<Point>(),
            &NetlogoInternalType::HEADING => TypeId::of::<Heading>(),
            &NetlogoInternalType::COLOR => TypeId::of::<Color>(),
            _ => panic!("Unknown NetLogo internal type: {:?}", value),
        }
    }
}
