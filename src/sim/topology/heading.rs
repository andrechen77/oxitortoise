use std::ops::AddAssign;

use crate::{
    sim::value::{self, Float},
    util::rng::NextInt,
};

use super::CoordFloat;

/// A heading. This is a floating point value representing some 2D angle in
/// degrees, and is guaranteed to be in the range of 0.0..360.0. 0 represents
/// a heading of north, and the heading increases clockwise. This is different
/// from the more popular convention of 0 meaning east, and angles increasing
/// counterclockwise.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Heading(f64);

impl Eq for Heading {}

/// The value at which a heading wraps back around to 0. This is not a valid
/// value for a heading.
const HEADING_MAX: f64 = 360.0;

impl Heading {
    /// The dx corresponding to this heading, which is the sine of the angle.
    pub fn dx(self) -> CoordFloat {
        self.0.to_radians().sin()
    }

    /// The dy corresponding to this heading, which is the cosine of the angle.
    pub fn dy(self) -> CoordFloat {
        self.0.to_radians().cos()
    }

    pub fn dx_and_dy(self) -> (CoordFloat, CoordFloat) {
        self.0.to_radians().sin_cos()
    }

    pub fn random<R: NextInt + ?Sized>(rng: &mut R) -> Self {
        Heading(rng.next_int(360) as f64)
    }

    pub fn to_float(self) -> Float {
        Float::new(self.0)
    }
}

impl AddAssign<value::Float> for Heading {
    fn add_assign(&mut self, rhs: value::Float) {
        self.0 += rhs.get();
        self.0 %= HEADING_MAX;
    }
}

impl From<Float> for Heading {
    fn from(value: Float) -> Self {
        Heading(value.get() % HEADING_MAX)
    }
}
