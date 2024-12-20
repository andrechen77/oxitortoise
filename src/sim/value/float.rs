use derive_more::derive::{
    Add, AddAssign, Display, Div, DivAssign, From, Mul, MulAssign, Neg, Sub, SubAssign,
};

use crate::sim::{color::Color, topology::CoordInt, turtle::TurtleWho};

/// A double-precision floating-point number which is guaranteed to be finite
/// (not Infinity or NaN).
#[derive(Debug, Display, Default, Clone, Copy, From, PartialEq, PartialOrd)] // TODO also ord and eq
#[derive(Neg, Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign)] // TODO how to keep guaranteeing that the result is finite?
#[mul(forward)]
#[div(forward)]
#[repr(transparent)]
pub struct Float(f64);

impl Float {
    pub const fn new(value: f64) -> Self {
        debug_assert!(value.is_finite());
        Self(value)
    }

    pub const fn get(&self) -> f64 {
        self.0
    }
}

impl From<CoordInt> for Float {
    fn from(value: CoordInt) -> Self {
        Float(value as f64)
    }
}

impl From<Color> for Float {
    fn from(value: Color) -> Self {
        value.to_float()
    }
}

impl From<TurtleWho> for Float {
    fn from(value: TurtleWho) -> Self {
        Float(value.0 as f64)
    }
}

impl Float {
    /// Extracts an `u64` from the value. If the value is a fractional value, it
    /// is rounded down. Negative values are truncated to zero.
    pub fn to_u64_round_to_zero(&self) -> u64 {
        self.0 as u64
    }
}
