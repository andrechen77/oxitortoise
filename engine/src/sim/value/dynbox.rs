use crate::sim::{patch::PatchId, turtle::TurtleId, value::NetlogoInternalType};

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
pub struct DynBox(u64);

const NAN_BASE: u64 = 0x7FF8000000000000;

#[derive(Debug)]
pub enum UnpackedDynBox {
    Float(f64),
    Int(i64),
    Bool(bool),
    Turtle(TurtleId),
    Patch(PatchId),
    Link(LinkId),
    // TODO make this a pointer to a value in storage
    Other(u64),
}

impl DynBox {
    /// A `DynBox` representing a numeric value of 0. This works because the
    /// all-zero bit pattern is a non-NaN value that represents +0.0.
    pub const ZERO: Self = Self(0u64);

    pub const FALSE: Self = Self(NAN_BASE | 0b001 << 48 | 0);
    pub const TRUE: Self = Self(NAN_BASE | 0b001 << 48 | 1);

    pub fn unpack(&self) -> UnpackedDynBox {
        let float = f64::from_bits(self.0);
        if !float.is_nan() {
            return UnpackedDynBox::Float(float);
        }

        let tag = (self.0 >> 48) & 0b111;
        let value = self.0 & 0xFFFFFFFFFFFF;

        match tag {
            0b000 => UnpackedDynBox::Int(value as i64), // TODO sign extend
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
}

impl std::fmt::Debug for DynBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DynBox({:?})", self.unpack())
    }
}

impl UnpackedDynBox {
    pub fn ty(&self) -> NetlogoInternalType {
        match self {
            UnpackedDynBox::Bool(_) => NetlogoInternalType::BOOLEAN,
            UnpackedDynBox::Int(_) => NetlogoInternalType::INTEGER,
            UnpackedDynBox::Float(_) => NetlogoInternalType::FLOAT,
            UnpackedDynBox::Turtle(_) => NetlogoInternalType::TURTLE_ID,
            UnpackedDynBox::Patch(_) => NetlogoInternalType::PATCH_ID,
            UnpackedDynBox::Link(_) => todo!("add link id"),
            UnpackedDynBox::Other(_) => todo!("match on the inner type"),
        }
    }
}

impl Clone for UnpackedDynBox {
    fn clone(&self) -> Self {
        match self {
            UnpackedDynBox::Float(value) => UnpackedDynBox::Float(*value),
            UnpackedDynBox::Int(value) => UnpackedDynBox::Int(*value),
            UnpackedDynBox::Bool(value) => UnpackedDynBox::Bool(*value),
            UnpackedDynBox::Turtle(value) => UnpackedDynBox::Turtle(*value),
            _ => unimplemented!(),
        }
    }
}

// TODO implement links
#[derive(Debug)]
pub struct LinkId;
