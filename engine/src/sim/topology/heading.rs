use std::ops::{Add, AddAssign};

use super::CoordFloat;
use crate::{
    sim::value::{self, NlFloat},
    util::{
        reflection::{ConcreteTy, ConstTypeName, Reflect, TypeInfo, TypeInfoOptions},
        rng::Rng,
    },
};

/// A heading. This is a floating point value representing some 2D angle in
/// degrees, and is guaranteed to be in the range of 0.0..360.0. 0 represents
/// a heading of north, and the heading increases clockwise. This is different
/// from the more popular convention of 0 meaning east, and angles increasing
/// counterclockwise.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[repr(transparent)]
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

    pub fn random<R: Rng + ?Sized>(rng: &mut R) -> Self {
        Heading(rng.next_int(360) as f64)
    }

    pub fn to_float(self) -> NlFloat {
        NlFloat::new(self.0)
    }
}

impl Add<value::NlFloat> for Heading {
    type Output = Heading;

    fn add(self, rhs: value::NlFloat) -> Self::Output {
        Heading((self.0 + rhs.get()) % HEADING_MAX)
    }
}

impl AddAssign<value::NlFloat> for Heading {
    fn add_assign(&mut self, rhs: value::NlFloat) {
        *self = *self + rhs;
    }
}

impl From<NlFloat> for Heading {
    fn from(value: NlFloat) -> Self {
        Heading(value.get() % HEADING_MAX)
    }
}

static HEADING_TYPE_INFO: TypeInfo = TypeInfo::new_copy::<Heading>(TypeInfoOptions {
    is_zeroable: true,
    mem_repr: Some(&[(0, lir::MemOpType::F64)]),
});

impl ConstTypeName for Heading {
    const TYPE_NAME: &'static str = "Heading";
}

unsafe impl Reflect for Heading {
    const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&HEADING_TYPE_INFO);
}
