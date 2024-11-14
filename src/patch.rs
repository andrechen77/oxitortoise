use crate::topology::Topology;

#[derive(Debug)]
pub struct Patches {
    /// The patches in the world, stored in row-major order. The first row
    /// contains the patches with the highest `pycor`, and the first column
    /// contains the patches with the lowest `pxcor`.
    patches: Vec<Patch>,
    /// The topology of the world.
    topology: Topology,
}

impl Patches {
    pub fn new(topology: Topology) -> Self {
        let Topology {
            world_width,
            world_height,
        } = topology;
        let patches = (0..world_width * world_height).map(|_| Patch {}).collect();
        Self { patches, topology }
    }

    pub fn clear_all_patches(&mut self) {
        // TODO keep this up to date with new patch variables. reset all
        // variables to their default values
    }
}

#[derive(Debug)]
pub struct Patch {
    // TODO some way of tracking what turtles are on this patch.
}
