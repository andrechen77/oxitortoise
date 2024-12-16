use std::sync::OnceLock;

use crate::{sim::value::Float, util::rng::NextInt};

/// A NetLogo color. This is a floating point value guaranteed to be in the
/// range 0.0..140.0. Values with more than one decimal place of precision are
/// remembered with that much precision, even though it doesn't matter for
/// rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color(f64);

impl Color {
    pub const BLACK: Color = Color(0.0);

    pub const GRAY: Color = Color(5.0);
    pub const RED: Color = Color(15.0);
    pub const ORANGE: Color = Color(25.0);
    pub const BROWN: Color = Color(35.0);
    pub const YELLOW: Color = Color(45.0);
    pub const LIME: Color = Color(55.0);
    pub const GREEN: Color = Color(65.0);
    pub const TURQUOISE: Color = Color(75.0);
    pub const CYAN: Color = Color(85.0);
    pub const SKY: Color = Color(95.0);
    pub const BLUE: Color = Color(105.0);
    pub const VIOLET: Color = Color(115.0);
    pub const MAGENTA: Color = Color(125.0);
    pub const PINK: Color = Color(135.0);

    pub fn to_darkest_shade(self) -> Color {
        Color((self.0 / SHADE_RANGE).floor() * SHADE_RANGE)
    }

    pub fn to_float(self) -> Float {
        Float::new(self.0)
    }
}

impl From<Float> for Color {
    fn from(value: Float) -> Self {
        Color(value.get() % COLOR_MAX)
    }
}

pub fn random_color(next_int: &mut dyn NextInt) -> Color {
    Color(next_int.next_int(base_colors().len() as i64) as f64)
}

const COLOR_MAX: f64 = 140.0;
const SHADE_RANGE: f64 = 10.0;

pub fn base_colors() -> &'static [Color] {
    static BASE_COLORS: OnceLock<Vec<Color>> = OnceLock::new();
    BASE_COLORS.get_or_init(|| {
        (0..((COLOR_MAX / 10.0) as i64))
            .map(|n| Color((n * 10 + 5) as f64))
            .collect()
    })
}

/// see https://ccl.northwestern.edu/netlogo/docs/dictionary.html#scale-color
// TODO how to ensure this works for extreme values that might cause overflow?
//
//  scale-color color number range1 range2
// Reports a shade of color proportional to the value of number.
// When range1 is less than or equal to range2, then the larger the number, the lighter the shade of color. However, if range2 is less than range1, the color scaling is inverted.
// Let min-range be the minimum of range1 and range2. If number is less than or equal to min-range, then the result is the same as if number was equal to min-range. Let max-range be the maximum of range1 and range2. If number is greater than max-range, then the result is the same as if number was equal to max-range.
// Note: for color shade is irrelevant, e.g. green and green + 2 are equivalent, and the same spectrum of colors will be used.
pub fn scale_color(color: Color, number: Float, range_start: Float, range_end: Float) -> Color {
    let range_start = range_start.get();
    let range_end = range_end.get();
    let number = number.get();
    let min_range = range_start.min(range_end);
    let max_range = range_start.max(range_end);
    let number = number.min(max_range).max(min_range);
    let proportion = (number - range_start) / (max_range - min_range);
    let color_value = color.to_darkest_shade().0 + proportion * SHADE_RANGE;
    Color(color_value)
}
