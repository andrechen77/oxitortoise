use std::ptr::NonNull;

use crate::util::reflection::{ConcreteTy, ConstTypeName, Reflect, TypeInfo, TypeInfoOptions};

use super::{BoxedAny, UnpackedAny};

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
pub struct PackedAny(f64);

// TODO is there a way we can preseve pointer provenance even though the pointers
// are stored as floats?

const NAN_BASE: u64 = 0x7FF8000000000000;

impl PackedAny {
    /// A [`PackedAny`] representing a numeric value of 0. This works because the
    /// all-zero bit pattern is a non-NaN value that represents +0.0.
    pub const ZERO: Self = Self(0.0);

    pub const FALSE: Self = Self(f64::from_bits(NAN_BASE | 0b001 << 48));
    pub const TRUE: Self = Self(f64::from_bits(NAN_BASE | 0b001 << 48 | 1));

    pub fn unpack(&self) -> UnpackedAny {
        if !self.0.is_nan() {
            return UnpackedAny::Float(self.0);
        }

        // TODO(wishlist) add constants for each tag variant
        let bits = self.0.to_bits();
        let tag = (bits >> 48) & 0b111;
        let value = bits & 0xFFFFFFFFFFFF;

        // TODO(mvp) complete all cases for unpacking a [`PackedAny`]
        match tag {
            0b001 => match value {
                0 => UnpackedAny::Bool(false),
                1 => UnpackedAny::Bool(true),
                other => unimplemented!(
                    "{:b} is not a recognized special value for [`PackedAny`].",
                    other
                ),
            },
            #[allow(unreachable_code)]
            0b100 => UnpackedAny::Turtle(todo!("reinterpret bits")),
            #[allow(unreachable_code)]
            0b101 => UnpackedAny::Patch(todo!("reinterpret bits")),
            #[allow(unreachable_code)]
            0b110 => UnpackedAny::Link(todo!("reinterpret bits")),
            0b111 => UnpackedAny::Other(
                // SAFETY: this pointer can only have come from a call to
                // `BoxedAny::as_raw` because there is no other assignment to
                // this variant that doesn't use `BoxedAny::as_raw`
                unsafe { BoxedAny::from_raw(NonNull::new_unchecked(value as *mut ConcreteTy)) },
            ),
            other => unimplemented!("{:b} is not a recognized tag for [`PackedAny`].", other),
        }
    }

    pub fn pack(value: UnpackedAny) -> Self {
        let (tag, value_bits) = match value {
            UnpackedAny::Float(value) => return PackedAny(value),
            UnpackedAny::Bool(value) => (0b001, value as u64),
            UnpackedAny::Nobody => todo!("TODO(mvp) implement nobody"),
            UnpackedAny::Turtle(_) => todo!("TODO(mvp) implement turtle"),
            UnpackedAny::Patch(_) => todo!("TODO(mvp) implement patch"),
            UnpackedAny::Link(_) => todo!("TODO(mvp) implement link"),
            UnpackedAny::Other(any) => {
                let raw = any.as_raw();
                let bits =
                    u64::try_from(raw.addr().get()).expect("ptr should be able to fit in 64 bits");
                (0b111, bits)
            }
        };
        assert!(value_bits >> 48 == 0, "value must fit in 48 bits to be packed");
        PackedAny(f64::from_bits(NAN_BASE | tag << 48 | value_bits))
    }

    pub fn and(self, rhs: Self) -> bool {
        self.unpack().and(rhs.unpack())
    }

    pub fn or(self, rhs: Self) -> bool {
        self.unpack().or(rhs.unpack())
    }
}

static PACKED_ANY_TYPE_INFO: TypeInfo = TypeInfo::new_drop::<PackedAny>(TypeInfoOptions {
    is_zeroable: true,
    mem_repr: Some(&[(0, lir::MemOpType::F64)]),
});

unsafe impl Reflect for PackedAny {
    const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&PACKED_ANY_TYPE_INFO);
}

impl ConstTypeName for PackedAny {
    const TYPE_NAME: &'static str = "PackedAny";
}

impl std::fmt::Debug for PackedAny {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PackedAny({:?})", self.unpack())
    }
}
