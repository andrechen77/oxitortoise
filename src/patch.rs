use std::ops::Index;

use crate::topology::{Point, Topology};

/// A reference to a patch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PatchId {
    // The index of the patch in the [`Patches`] struct.
    grid_index: usize,
}

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
            min_pxcor,
            max_pycor,
        } = topology;
        let mut patches = Vec::with_capacity((world_width * world_height) as usize);
        for j in 0..world_height {
            for i in 0..world_width {
                let x = (min_pxcor + i as i64) as f64;
                let y = (max_pycor - j as i64) as f64;
                patches.push(Patch {
                    location: Point { x, y },
                });
            }
        }
        Self { patches, topology }
    }

    pub fn patch_ids_iter(&self) -> impl Iterator<Item = PatchId> {
        (0..self.patches.len()).map(|i| PatchId { grid_index: i })
    }

    pub fn clear_all_patches(&mut self) {
        // TODO keep this up to date with new patch variables. reset all
        // variables to their default values
    }
}

impl Index<PatchId> for Patches {
    type Output = Patch;

    fn index(&self, index: PatchId) -> &Self::Output {
        &self.patches[index.grid_index]
    }
}

#[derive(Debug)]
pub struct Patch {
    location: Point,
    // TODO some way of tracking what turtles are on this patch.
}
