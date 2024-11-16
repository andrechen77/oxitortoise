use derive_more::derive::From;

use crate::topology::CoordInt;

/// A double-precision floating-point number which is guaranteed to be finite
/// (not Infinity or NaN).
#[derive(Debug, Clone, Copy, From, PartialEq, PartialOrd)] // TODO also ord and eq
pub struct Float(f64);

impl Float {
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    pub fn get(&self) -> f64 {
        self.0
    }
}

impl From<CoordInt> for Float {
    fn from(value: CoordInt) -> Self {
        Float(value as f64)
    }
}

impl Float {
    /// Extracts an `u64` from the value. If the value is a fractional value, it
    /// is rounded down. Negative values are truncated to zero.
    pub fn to_u64_round_to_zero(&self) -> u64 {
        self.0 as u64
    }
}
