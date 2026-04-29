use std::{fmt, mem::offset_of};

use macro_reflect::{MirReflect, reflect};
use reflection::{Reflect as _, mir};

use super::{patch::PatchId, value, value::NlFloat};
pub use heading::Heading;

pub mod diffuse;
mod heading;

/// The type used to refer to integer patch coordinates.
pub type CoordInt = i32;

/// The type used to refer to floating-point patch coordinates.
pub type CoordFloat = NlFloat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct PointInt {
    pub x: CoordInt,
    pub y: CoordInt,
}

impl fmt::Display for PointInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

// QUESTION currently we use double-NaN to represent a point that does not
// exist. (i.e. None). Consider if the codebase would benefit from non-nullable
// points and if so, make Points non-nullable and use a different type
// OptionPoint which allows NaN to be used for None.
#[derive(Debug, Clone, Copy, PartialEq, Default, MirReflect)]
#[repr(C)]
pub struct Point {
    #[mir_accessible]
    pub x: CoordFloat,
    #[mir_accessible]
    pub y: CoordFloat,
}

#[reflect(unsafe(is_zeroable), clone(copy))]
impl Reflect for Point {}

impl Point {
    pub const ORIGIN: Point = Point { x: NlFloat::new(0.0), y: NlFloat::new(0.0) };

    pub fn round_to_int(self) -> PointInt {
        PointInt { x: self.x.round() as CoordInt, y: self.y.round() as CoordInt }
    }

    pub fn mir_initialize_from_local(
        builder: &mut mir::FunctionBuilder,
        x: mir::LocalId,
        y: mir::LocalId,
    ) -> mir::LocalId {
        // this will light up if we change the fields without updating the function.
        // This helps us ensure that all fields are initialized
        #[allow(dead_code)]
        const {
            let _ = |s: Point| {
                let Point { x: _, y: _ } = s;
            };
        };

        let result_local =
            builder.create_local(mir::LocalDecl { debug_name: None, ty: Self::dyn_type() });

        builder.add_operation_with_dst(
            result_local.place().proj_field(offset_of!(Self, x)),
            mir::Operation::Operand(mir::PlaceOperand::Direct(x.place())),
        );
        builder.add_operation_with_dst(
            result_local.place().proj_field(offset_of!(Self, y)),
            mir::Operation::Operand(mir::PlaceOperand::Direct(y.place())),
        );

        builder.set_as_init(result_local);
        result_local
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl From<PointInt> for Point {
    fn from(point: PointInt) -> Self {
        Point { x: point.x.into(), y: point.y.into() }
    }
}

#[derive(Debug, Clone, Copy)]
// Specifies the topology of the world.
pub struct TopologySpec {
    /// The `pxcor` of the leftmost patch as an integer. Since `pxcor` describes
    /// the point at the center of the patch, the *actual* leftmost meaningful
    /// coordinate is min_pxcor - 0.5.
    pub min_pxcor: CoordInt,
    /// The `pycor` of the topmost patch. Since `pycor` describes the point at
    /// the center of the patch, the *actual* topmist meaningful coordinate
    /// is max_pycor + 0.5.
    pub max_pycor: CoordInt,
    /// The width of the world in patches. Must be positive.
    pub patches_width: CoordInt, // TODO(wishlist) make this guaranteed NonZero
    /// The height of the world in patches. Must be positive.
    pub patches_height: CoordInt,
    /// Whether the world wraps around in the x direction.
    pub wrap_x: bool,
    /// Whether the world wraps around in the y direction.
    pub wrap_y: bool,
}

impl TopologySpec {
    #[inline(always)]
    pub fn num_patches(&self) -> u32 {
        self.patches_width as u32 * self.patches_height as u32
    }

    #[inline(always)]
    pub fn max_pxcor(&self) -> CoordInt {
        self.min_pxcor + self.patches_width - 1
    }

    #[inline(always)]
    pub fn min_pycor(&self) -> CoordInt {
        self.max_pycor - self.patches_height + 1
    }

    /// Assumes the patch exists, and returns nonsense if it doesn't.
    #[inline(always)]
    pub fn patch_at(&self, point: PointInt) -> PatchId {
        let width = self.patches_width;
        let max_pycor = self.max_pycor;
        let min_pxcor = self.min_pxcor;
        let i = (max_pycor - point.y) * width + (point.x - min_pxcor);
        PatchId(i as u32)
    }
}

#[derive(Debug, Clone, MirReflect)]
pub struct Topology {
    pub spec: TopologySpec,
    // QUESTION the max coordinates currently go all the way to the edge of the
    // border patches (e.g. 16.5 instead of 16.0), but the max-pxcor and
    // max-pycor reporters only report the center. See if this needs to be
    // changed based on which version is used more often.
    #[mir_accessible]
    pub min_x: CoordFloat,
    #[mir_accessible]
    pub max_x: CoordFloat,
    #[mir_accessible]
    pub world_width: CoordFloat,
    #[mir_accessible]
    pub min_y: CoordFloat,
    #[mir_accessible]
    pub max_y: CoordFloat,
    #[mir_accessible]
    pub world_height: CoordFloat,
}

#[reflect(special_mir_type)]
impl Reflect for Topology {}

impl Topology {
    pub fn new(spec: TopologySpec) -> Self {
        Self {
            spec,
            min_x: NlFloat::from(spec.min_pxcor) - NlFloat::from(0.5),
            max_x: NlFloat::from(spec.min_pxcor + spec.patches_width) - NlFloat::from(0.5),
            world_width: NlFloat::from(spec.patches_width + 1),
            min_y: NlFloat::from(spec.max_pycor - spec.patches_height) + NlFloat::from(0.5),
            max_y: NlFloat::from(spec.max_pycor) + NlFloat::from(0.5),
            world_height: NlFloat::from(spec.patches_height + 1),
        }
    }

    #[inline(always)]
    pub fn spec(&self) -> &TopologySpec {
        &self.spec
    }

    #[inline(always)]
    pub fn min_pxcor(&self) -> CoordInt {
        self.spec.min_pxcor
    }

    #[inline(always)]
    pub fn max_pxcor(&self) -> CoordInt {
        self.spec.max_pxcor()
    }

    #[inline(always)]
    pub fn patches_width(&self) -> CoordInt {
        self.spec.patches_width
    }

    #[inline(always)]
    pub fn min_pycor(&self) -> CoordInt {
        self.spec.min_pycor()
    }

    #[inline(always)]
    pub fn max_pycor(&self) -> CoordInt {
        self.spec.max_pycor
    }

    #[inline(always)]
    pub fn patches_height(&self) -> CoordInt {
        self.spec.patches_height
    }

    #[inline(always)]
    pub fn num_patches(&self) -> u32 {
        self.spec.num_patches()
    }

    #[inline(always)]
    pub fn wrap_x(&self) -> bool {
        self.spec.wrap_x
    }

    #[inline(always)]
    pub fn wrap_y(&self) -> bool {
        self.spec.wrap_y
    }

    #[inline(always)]
    pub fn min_x(&self) -> CoordFloat {
        self.min_x
    }

    #[inline(always)]
    pub fn max_x(&self) -> CoordFloat {
        self.max_x
    }

    #[inline(always)]
    pub fn world_width(&self) -> CoordFloat {
        self.world_width
    }

    #[inline(always)]
    pub fn min_y(&self) -> CoordFloat {
        self.min_y
    }

    #[inline(always)]
    pub fn max_y(&self) -> CoordFloat {
        self.max_y
    }

    #[inline(always)]
    pub fn world_height(&self) -> CoordFloat {
        self.world_height
    }

    /// Derives `max-pxcor` from a `Topology`.
    pub fn mir_project_max_pxcor(topology: mir::Place) -> mir::Place {
        topology.proj_field(offset_of!(Topology, max_x))
    }

    /// Derives `max-pycor` from a `Topology`.
    pub fn mir_project_max_pycor(topology: mir::Place) -> mir::Place {
        topology.proj_field(offset_of!(Topology, max_y))
    }

    /// Returns the `PatchId` of the patch at the given position. Assumes that
    /// the position is inside the world boundaries without having to wrap,
    /// otherwise the PatchId returned will be nonsense.
    #[inline]
    pub fn patch_at(&self, point: PointInt) -> PatchId {
        self.spec.patch_at(point)
    }

    /// Takes a point and returns its wrapped equivalent. If the point is
    /// outside the world boundaries and does not wrap, returns `None`.
    pub fn wrap_point(&self, mut point: Point) -> Option<Point> {
        if self.wrap_x() {
            point.x = wrap_coordinate(point.x, self.min_x(), self.world_width());
        } else if point.x < self.min_x() || point.x >= self.max_x() {
            return None;
        }
        if self.wrap_y() {
            point.y = wrap_coordinate(point.y, self.min_y(), self.world_height());
        } else if point.y < self.min_y() || point.y >= self.max_y() {
            return None;
        }
        Some(point)
    }

    pub fn offset_one_by_heading(&self, point: Point, heading: Heading) -> Option<Point> {
        let (dx, dy) = heading.dx_and_dy();
        let new_x = point.x + dx;
        let new_y = point.y + dy;
        self.wrap_point(Point { x: new_x, y: new_y })
    }

    pub fn offset_distance_by_heading(
        &self,
        point: Point,
        heading: Heading,
        distance: value::NlFloat,
    ) -> Option<Point> {
        let (dx, dy) = heading.dx_and_dy();
        let new_x = point.x + dx * distance;
        let new_y = point.y + dy * distance;
        self.wrap_point(Point { x: new_x, y: new_y })
    }
}

fn wrap_coordinate(coord: CoordFloat, min: CoordFloat, len: CoordFloat) -> CoordFloat {
    // the remainder has an absolute value less than `len`, and is positive if
    // coord > min and negative if coord < min
    let len = len.get();
    let remainder = (coord - min).get() % len;
    let offset_from_min = if remainder < 0.0 { len + remainder } else { remainder };
    NlFloat::new(*min + offset_from_min)
}

pub fn euclidean_distance_no_wrap(a: Point, b: Point) -> value::NlFloat {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    value::NlFloat::new((dx * dx + dy * dy).sqrt())
}
