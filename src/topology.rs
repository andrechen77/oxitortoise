#[derive(Debug, Clone)]
pub struct Topology {
    /// The `pxcor` of the leftmost patch. Since `pxcor` describes the point at
    /// the center of the patch, the *actual* leftmost meaningful coordinate
    /// is min_pxcor - 0.5.
    pub min_pxcor: i64,
    /// The `pycor` of the topmost patch. Since `pycor` describes the point at
    /// the center of the patch, the *actual* topmist meaningful coordinate
    /// is max_pycor + 0.5.
    pub max_pycor: i64,
    pub world_width: u64,
    pub world_height: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const ORIGIN: Point = Point { x: 0.0, y: 0.0 };
}

pub fn euclidean_distance_unwrapped(a: Point, b: Point) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}
