use std::sync::OnceLock;

use crate::rng::NextInt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color(pub f64); // do we need this much precision? does it matter?

pub static COLOR_MAX: Color = Color(140.0);

pub fn random_color(next_int: &mut dyn NextInt) -> Color {
    Color(next_int.next_int(base_colors().len() as i32).into())
}

pub fn base_colors() -> &'static [Color] {
    static BASE_COLORS: OnceLock<Vec<Color>> = OnceLock::new();
    BASE_COLORS.get_or_init(|| {
        (0..((COLOR_MAX.0 / 10.0) as i64))
            .map(|n| Color((n * 10 + 5) as f64))
            .collect()
    })
}

pub static RED: Color = Color(15.0);
