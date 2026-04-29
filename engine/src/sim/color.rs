use reflection::mir;

use crate::{sim::value::NlFloat, util::rng::Rng};

pub const BLACK: NlFloat = NlFloat::new(0.0);
pub const GRAY: NlFloat = NlFloat::new(5.0);
pub const RED: NlFloat = NlFloat::new(15.0);
pub const ORANGE: NlFloat = NlFloat::new(25.0);
pub const BROWN: NlFloat = NlFloat::new(35.0);
pub const YELLOW: NlFloat = NlFloat::new(45.0);
pub const LIME: NlFloat = NlFloat::new(55.0);
pub const GREEN: NlFloat = NlFloat::new(65.0);
pub const TURQUOISE: NlFloat = NlFloat::new(75.0);
pub const CYAN: NlFloat = NlFloat::new(85.0);
pub const SKY: NlFloat = NlFloat::new(95.0);
pub const BLUE: NlFloat = NlFloat::new(105.0);
pub const VIOLET: NlFloat = NlFloat::new(115.0);
pub const MAGENTA: NlFloat = NlFloat::new(125.0);
pub const PINK: NlFloat = NlFloat::new(135.0);

const BASE_COLORS: [NlFloat; 14] = [
    GRAY, RED, ORANGE, BROWN, YELLOW, LIME, GREEN, TURQUOISE, CYAN, SKY, BLUE, VIOLET, MAGENTA,
    PINK,
];

pub fn random(next_int: &mut dyn Rng) -> NlFloat {
    NlFloat::new(next_int.next_int(BASE_COLORS.len() as i64) as f64)
}

pub fn to_darkest_shade(color: NlFloat) -> NlFloat {
    NlFloat::new((color.get() / SHADE_RANGE).floor() * SHADE_RANGE)
}

pub fn wrap_color(float: NlFloat) -> NlFloat {
    NlFloat::new(float.get() % COLOR_MAX)
}

pub fn mir_wrap_color(builder: &mut mir::FunctionBuilder, color: mir::Place) -> mir::LocalId {
    let operation = mir::Operation::CallHostFunction {
        function: &wrap_color::FN_INFO,
        args: vec![mir::PlaceOperand::Direct(color)],
    };
    builder.add_operation(None, operation)
}

mod wrap_color {
    use super::*;

    use reflection::{Reflect as _, mir::HostFunctionInfo};

    pub static FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "wrap_color",
        parameter_types: &[NlFloat::STATIC_TYPE],
        return_type: NlFloat::STATIC_TYPE,
        link_name: "wrap_color",
        link_addr: call as *const u8,
    };

    pub fn call(color: NlFloat) -> NlFloat {
        super::wrap_color(color)
    }
}

const COLOR_MAX: f64 = 140.0;
const SHADE_RANGE: f64 = 10.0;

/// see https://ccl.northwestern.edu/netlogo/docs/dictionary.html#scale-color
// FIXME ensure this works for extreme values that might cause overflow
//
//  scale-color color number range1 range2
// Reports a shade of color proportional to the value of number.
// When range1 is less than or equal to range2, then the larger the number, the lighter the shade of color. However, if range2 is less than range1, the color scaling is inverted.
// Let min-range be the minimum of range1 and range2. If number is less than or equal to min-range, then the result is the same as if number was equal to min-range. Let max-range be the maximum of range1 and range2. If number is greater than max-range, then the result is the same as if number was equal to max-range.
// Note: for color shade is irrelevant, e.g. green and green + 2 are equivalent, and the same spectrum of colors will be used.
pub fn scale_color(
    color: NlFloat,
    number: NlFloat,
    range_start: NlFloat,
    range_end: NlFloat,
) -> NlFloat {
    let range_start = range_start.get();
    let range_end = range_end.get();
    let number = number.get();

    // FIXME handle case where NaNs might show up in the calculation.

    // determine where in the the range the number is
    let proportion = (number - range_start) / (range_end - range_start);

    // scale the proportion to the color range to determine the amount to add to
    // the darkest shade of the color to get the desired color
    let color_shade_offset = proportion * SHADE_RANGE;
    // clamp the offset to be within [0, SHADE_RANGE). the value of 9.9999 was
    // copied from the original Tortoise, but it must be some number less than
    // SHADE_RANGE
    // https://github.com/NetLogo/Tortoise/blob/master/engine/src/main/coffee/engine/core/colormodel.coffee#L267
    let color_shade_offset = color_shade_offset.clamp(0.0, SHADE_RANGE - 0.0001);

    // add the offset to the darkest shade of the color to get the correct shade
    let color_value = to_darkest_shade(color) + NlFloat::new(color_shade_offset);

    color_value
}
