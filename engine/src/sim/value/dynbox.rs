use std::{
    cmp::Ordering,
    ops::{Add, Div, Mul, Sub},
};

use crate::{
    sim::{
        patch::PatchId,
        turtle::TurtleId,
        value::{NlBool, NlFloat},
    },
    util::reflection::{ConcreteTy, Reflect, TypeInfo, TypeInfoOptions},
};

/// An 8-byte box that can hold a value of any type.
///
/// A non-NaN floating point value is stored as-is in the entire 64 bits. This
/// type does not allow NaN floating point values. A quiet NaN floating point
/// bit pattern represents any type other than a float, with the bottom 51
/// bits of the significand being the payload. Of these 51 bits, the top 3 bits
/// are a tag, and the bottom 48 bytes are the value.
///
/// | format | description |
/// | --- | --- |
/// | non-NaN flaot | 64-bit floating point value |
/// | signaling NaN | impossible |
/// | quiet NaN, MSB `0b000` | bottom 48 bits are a signed integer |
/// | quiet NaN, MSB `0b001` | special values, see below |
/// | quiet NaN, MSB `0b100` | bottom 48 bits are a [`TurtleId`] |
/// | quiet NaN, MSB `0b101` | bottom 32 bits are a [`PatchId`] |
/// | quiet NaN, MSB `0b110` | bottom 48 bits are a [`LinkId`] |
/// | quiet NaN, MSB `0b111` | bottom 48 bits are a pointer to any other value |
///
/// Special sentinel values:
/// | lower 48 bits | description |
/// | --- | --- |
/// | 0 | false |
/// | 1 | true |
#[derive(Clone)]
#[repr(transparent)]
pub struct DynBox(u64);

const NAN_BASE: u64 = 0x7FF8000000000000;

#[derive(Debug)]
pub enum UnpackedDynBox {
    Float(f64),
    Bool(bool),
    Nobody,
    Turtle(TurtleId),
    Patch(PatchId),
    Link(LinkId),
    // TODO(mvp) make this a pointer to a value in storage
    // see mention of a "boxed representation" the parent module
    Other(u64),
}

impl DynBox {
    /// A `DynBox` representing a numeric value of 0. This works because the
    /// all-zero bit pattern is a non-NaN value that represents +0.0.
    pub const ZERO: Self = Self(0u64);

    pub const FALSE: Self = Self(NAN_BASE | 0b001 << 48);
    pub const TRUE: Self = Self(NAN_BASE | 0b001 << 48 | 1);

    pub fn unpack(&self) -> UnpackedDynBox {
        let float = f64::from_bits(self.0);
        if !float.is_nan() {
            return UnpackedDynBox::Float(float);
        }

        // TODO(wishlist) add constants for each tag variant
        let tag = (self.0 >> 48) & 0b111;
        let value = self.0 & 0xFFFFFFFFFFFF;

        // TODO(mvp) complete all cases for unpacking a DynBox
        match tag {
            0b001 => match value {
                0 => UnpackedDynBox::Bool(false),
                1 => UnpackedDynBox::Bool(true),
                _ => unimplemented!("This is not a recognized special value for DynBox."),
            },
            #[allow(unreachable_code)]
            0b100 => UnpackedDynBox::Turtle(todo!("reinterpret bits")),
            #[allow(unreachable_code)]
            0b101 => UnpackedDynBox::Patch(todo!("reinterpret bits")),
            #[allow(unreachable_code)]
            0b110 => UnpackedDynBox::Link(todo!("reinterpret bits")),
            0b111 => UnpackedDynBox::Other(value),
            _ => unimplemented!("This is not a recognized tag for DynBox."),
        }
    }

    pub fn pack(value: UnpackedDynBox) -> Self {
        match value {
            UnpackedDynBox::Float(value) => DynBox(value.to_bits()),
            UnpackedDynBox::Bool(value) => DynBox(NAN_BASE | 0b001 << 48 | (value as u64)),
            UnpackedDynBox::Nobody => todo!("TODO(mvp) implement nobody"),
            UnpackedDynBox::Turtle(value) => todo!("TODO(mvp) implement turtle"),
            UnpackedDynBox::Patch(value) => todo!("TODO(mvp) implement patch"),
            UnpackedDynBox::Link(value) => todo!("TODO(mvp) implement link"),
            UnpackedDynBox::Other(value) => todo!("TODO(mvp) implement other"),
        }
    }

    pub fn and(self, rhs: Self) -> bool {
        self.unpack().and(rhs.unpack())
    }

    pub fn or(self, rhs: Self) -> bool {
        self.unpack().or(rhs.unpack())
    }
}

static DYN_BOX_TYPE_INFO: TypeInfo = TypeInfo::new::<DynBox>(TypeInfoOptions {
    debug_name: "DynBox",
    is_zeroable: true,
    mem_repr: Some(&[(0, lir::MemOpType::F64)]),
});

impl Reflect for DynBox {
    const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&DYN_BOX_TYPE_INFO);
}

impl std::fmt::Debug for DynBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DynBox({:?})", self.unpack())
    }
}

impl UnpackedDynBox {
    pub fn ty(&self) -> ConcreteTy {
        match self {
            UnpackedDynBox::Bool(_) => NlBool::CONCRETE_TY,
            UnpackedDynBox::Float(_) => NlFloat::CONCRETE_TY,
            UnpackedDynBox::Nobody => NlBool::CONCRETE_TY,
            UnpackedDynBox::Turtle(_) => TurtleId::CONCRETE_TY,
            UnpackedDynBox::Patch(_) => PatchId::CONCRETE_TY,
            UnpackedDynBox::Link(_) => todo!("add link id"),
            UnpackedDynBox::Other(_) => todo!("match on the inner type"),
        }
    }

    pub fn and(self, rhs: Self) -> bool {
        match (self, rhs) {
            (UnpackedDynBox::Bool(lhs), UnpackedDynBox::Bool(rhs)) => lhs && rhs,
            (lhs, rhs) => unimplemented!("Anding {:?} and {:?} is not supported", lhs, rhs),
        }
    }

    pub fn or(self, rhs: Self) -> bool {
        match (self, rhs) {
            (UnpackedDynBox::Bool(lhs), UnpackedDynBox::Bool(rhs)) => lhs || rhs,
            (lhs, rhs) => unimplemented!("Oring {:?} and {:?} is not supported", lhs, rhs),
        }
    }
}

impl Clone for UnpackedDynBox {
    fn clone(&self) -> Self {
        match self {
            UnpackedDynBox::Float(value) => UnpackedDynBox::Float(*value),
            UnpackedDynBox::Bool(value) => UnpackedDynBox::Bool(*value),
            UnpackedDynBox::Turtle(value) => UnpackedDynBox::Turtle(*value),
            _ => unimplemented!(),
        }
    }
}

impl Add for UnpackedDynBox {
    type Output = UnpackedDynBox;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (UnpackedDynBox::Float(lhs), UnpackedDynBox::Float(rhs)) => {
                UnpackedDynBox::Float(lhs + rhs)
            }
            (lhs, rhs) => unimplemented!("Adding {:?} and {:?} is not supported", lhs, rhs),
        }
    }
}

impl Sub for UnpackedDynBox {
    type Output = UnpackedDynBox;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (UnpackedDynBox::Float(lhs), UnpackedDynBox::Float(rhs)) => {
                UnpackedDynBox::Float(lhs - rhs)
            }
            (lhs, rhs) => unimplemented!("Subtracting {:?} and {:?} is not supported", lhs, rhs),
        }
    }
}

impl Mul for UnpackedDynBox {
    type Output = UnpackedDynBox;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (UnpackedDynBox::Float(lhs), UnpackedDynBox::Float(rhs)) => {
                UnpackedDynBox::Float(lhs * rhs)
            }
            (lhs, rhs) => unimplemented!("Multiplying {:?} and {:?} is not supported", lhs, rhs),
        }
    }
}

impl Div for UnpackedDynBox {
    type Output = UnpackedDynBox;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (UnpackedDynBox::Float(lhs), UnpackedDynBox::Float(rhs)) => {
                UnpackedDynBox::Float(lhs / rhs)
            }
            (lhs, rhs) => unimplemented!("Dividing {:?} and {:?} is not supported", lhs, rhs),
        }
    }
}

impl PartialEq for UnpackedDynBox {
    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (UnpackedDynBox::Float(lhs), UnpackedDynBox::Float(rhs)) => lhs == rhs,
            (lhs, rhs) => unimplemented!("Comparing {:?} and {:?} is not supported", lhs, rhs),
        }
    }
}

impl PartialOrd for UnpackedDynBox {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        match (self, rhs) {
            (UnpackedDynBox::Float(lhs), UnpackedDynBox::Float(rhs)) => lhs.partial_cmp(rhs),
            (lhs, rhs) => unimplemented!("Comparing {:?} and {:?} is not supported", lhs, rhs),
        }
    }
}

impl Add for DynBox {
    type Output = DynBox;
    fn add(self, rhs: Self) -> Self::Output {
        DynBox::pack(self.unpack() + rhs.unpack())
    }
}

impl Sub for DynBox {
    type Output = DynBox;
    fn sub(self, rhs: Self) -> Self::Output {
        DynBox::pack(self.unpack() - rhs.unpack())
    }
}

impl Mul for DynBox {
    type Output = DynBox;
    fn mul(self, rhs: Self) -> Self::Output {
        DynBox::pack(self.unpack() * rhs.unpack())
    }
}

impl Div for DynBox {
    type Output = DynBox;
    fn div(self, rhs: Self) -> Self::Output {
        DynBox::pack(self.unpack() / rhs.unpack())
    }
}

impl PartialEq for DynBox {
    fn eq(&self, rhs: &Self) -> bool {
        self.unpack() == rhs.unpack()
    }
}

impl PartialOrd for DynBox {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        self.unpack().partial_cmp(&rhs.unpack())
    }
}

// placeholder; remove once links are implemented
#[derive(Debug)]
pub struct LinkId;
