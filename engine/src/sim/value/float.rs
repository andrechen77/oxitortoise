use derive_more::derive::{
    Add, AddAssign, Display, Div, DivAssign, From, Mul, MulAssign, Neg, Sub, SubAssign,
};

use crate::{
    sim::{color::Color, topology::CoordInt, turtle::TurtleWho},
    util::reflection::{ConcreteTy, ConstTypeName, Reflect, TypeInfo, TypeInfoOptions},
};

/// A double-precision floating-point number which is guaranteed to be finite
/// (not Infinity or NaN).
#[derive(Debug, Display, Default, Clone, Copy, From, PartialEq, PartialOrd)] // TODO(mvp) implement Ord and Eq
#[derive(Neg, Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign)] // FIXME these impls don't guarantee that the result is finite
#[mul(forward)]
#[div(forward)]
#[repr(transparent)]
pub struct NlFloat(f64);

impl NlFloat {
    pub const fn new(value: f64) -> Self {
        debug_assert!(value.is_finite());
        Self(value)
    }

    pub const fn get(self) -> f64 {
        self.0
    }
}

static NL_FLOAT_TYPE_INFO: TypeInfo = TypeInfo::new::<NlFloat>(TypeInfoOptions {
    is_zeroable: true,
    mem_repr: Some(&[(0, lir::MemOpType::F64)]),
});

unsafe impl Reflect for NlFloat {
    const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&NL_FLOAT_TYPE_INFO);
}

impl ConstTypeName for NlFloat {
    const TYPE_NAME: &'static str = "NlFloat";
}

impl From<CoordInt> for NlFloat {
    fn from(value: CoordInt) -> Self {
        NlFloat(value as f64)
    }
}

impl From<Color> for NlFloat {
    fn from(value: Color) -> Self {
        value.to_float()
    }
}

impl From<TurtleWho> for NlFloat {
    fn from(value: TurtleWho) -> Self {
        NlFloat(value.0 as f64)
    }
}

impl NlFloat {
    /// Extracts an `u64` from the value. If the value is a fractional value, it
    /// is rounded down. Negative values are truncated to zero.
    pub fn to_u64_round_to_zero(&self) -> u64 {
        self.0 as u64
    }
}
