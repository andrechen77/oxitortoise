use derive_more::{
    Deref,
    derive::{Add, AddAssign, Display, Div, DivAssign, From, Mul, MulAssign, Neg, Sub, SubAssign},
};
use macro_reflect::{ReflectComponents, reflect};

use crate::sim::{color::Color, topology::CoordInt, turtle::TurtleWho};

/// A double-precision floating-point number which is guaranteed to be finite
/// (not Infinity or NaN).
#[derive(Debug, Display, Default, Clone, Copy, From, PartialEq, PartialOrd, ReflectComponents)]
// TODO(mvp) implement Ord and Eq
// FIXME these impls don't guarantee that the result is finite. add similar
// changes to the compilation of arithmetic operations in the HIR
#[derive(Deref, Neg, Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign)]
#[mul(forward)]
#[div(forward)]
#[repr(transparent)]
pub struct NlFloat(f64);

#[reflect(unsafe(is_zeroable), clone(copy))]
impl Reflect for NlFloat {}

impl NlFloat {
    pub const fn new(value: f64) -> Self {
        debug_assert!(value.is_finite());
        Self(value)
    }

    pub const fn get(self) -> f64 {
        self.0
    }
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
        NlFloat(value.0)
    }
}

impl NlFloat {
    /// Extracts an `u64` from the value. If the value is a fractional value, it
    /// is rounded down. Negative values are truncated to zero.
    pub fn to_u64_round_to_zero(&self) -> u64 {
        self.0 as u64
    }
}
