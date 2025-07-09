use crate::sim::{
    patch::PatchId,
    topology::{self, Heading, Point, PointInt},
    value,
    world::World,
};

#[no_mangle]
#[inline(never)]
pub extern "C" fn distance_euclidean_no_wrap(a: Point, b: Point) -> value::Float {
    topology::euclidean_distance_no_wrap(a, b)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn min_pxcor(world: &World) -> value::Float {
    world.topology.min_pxcor().into()
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn max_pxcor(world: &World) -> value::Float {
    world.topology.max_pxcor().into()
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn min_pycor(world: &World) -> value::Float {
    world.topology.min_pycor().into()
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn max_pycor(world: &World) -> value::Float {
    world.topology.max_pycor().into()
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn offset_one_by_heading(
    world: &World,
    point: Point,
    heading: Heading,
) -> Option<Point> {
    world.topology.offset_one_by_heading(point, heading)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn offset_distance_by_heading(
    world: &World,
    point: Point,
    heading: Heading,
    distance: value::Float,
) -> Option<Point> {
    world
        .topology
        .offset_distance_by_heading(point, heading, distance)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn patch_at(world: &World, point: PointInt) -> PatchId {
    world.topology.patch_at(point)
}
