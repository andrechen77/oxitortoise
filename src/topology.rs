use std::fmt;

/// The type used to refer to integer patch coordinates.
pub type CoordInt = i32;

/// The type used to refer to floating-point patch coordinates.
pub type CoordFloat = f64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PointInt {
    pub x: CoordInt,
    pub y: CoordInt,
}

impl fmt::Display for PointInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: CoordFloat,
    pub y: CoordFloat,
}

impl Point {
    pub const ORIGIN: Point = Point { x: 0.0, y: 0.0 };
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl From<PointInt> for Point {
    fn from(point: PointInt) -> Self {
        Point {
            x: point.x as CoordFloat,
            y: point.y as CoordFloat,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Topology {
    /// The `pxcor` of the leftmost patch. Since `pxcor` describes the point at
    /// the center of the patch, the *actual* leftmost meaningful coordinate
    /// is min_pxcor - 0.5.
    pub min_pxcor: CoordInt,
    /// The `pycor` of the topmost patch. Since `pycor` describes the point at
    /// the center of the patch, the *actual* topmist meaningful coordinate
    /// is max_pycor + 0.5.
    pub max_pycor: CoordInt,
    /// The width of the world in patches. Must be positive.
    pub world_width: CoordInt, // TODO nonzero<>??
    /// The height of the world in patches. Must be positive.
    pub world_height: CoordInt,
}

pub fn euclidean_distance_unwrapped(a: Point, b: Point) -> CoordFloat {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}
