use std::ops::{Index, IndexMut};

use crate::sim::{
    agent_schema::AgentFieldDescriptor, topology::TopologySpec, value::Float, world::World,
};

/// Diffuses the value of the single variable in the specified buffer.
pub fn diffuse_8_single_variable_buffer(
    world: &mut World,
    patch_field: AgentFieldDescriptor,
    fraction: Float,
) {
    let old_values = world.patches.take_patch_values(patch_field);
    let new_values = world.patches.patch_field_as_mut_array::<Float>(patch_field);
    diffuse_8(world.topology.spec(), &old_values, new_values, fraction);
}

fn diffuse_8<R, W>(topology: &TopologySpec, old_values: &R, new_values: &mut W, fraction: Float)
where
    R: Index<usize, Output = Float> + ?Sized,
    W: IndexMut<usize, Output = Float> + ?Sized,
{
    let calculate_single_patch_value = |idx: usize, mine: Float, neighbors: [Float; 8]| {
        // the order in which we sum them is important, apparently
        // https://github.com/NetLogo/Tortoise/blob/master/engine/src/main/coffee/engine/core/topology/diffuser.coffee#L8
        let neighbors_sum = idiosyncratic_sum_8(neighbors);
        let new_val = mine + fraction * (neighbors_sum / Float::new(8.0) - mine);

        new_values[idx] = new_val;
    };

    run_with_neighbors8(topology, old_values, calculate_single_patch_value);
}

/// Runs the specified function for each patch, passing the function the index
/// of the patch, the value associated with the patch, and the values associated
/// with the patch's 8 neighbors. The neighbors are reported in the order of
/// E, N, S, W, NE, NW, SE, SW.
pub fn run_with_neighbors8<R, F>(topology: &TopologySpec, old_values: &R, mut f: F)
where
    R: Index<usize, Output = Float> + ?Sized,
    F: FnMut(usize, Float, [Float; 8]),
{
    let w = topology.patches_width as usize;
    let h = topology.patches_height as usize;

    // handle corners first
    {
        // NW corner (0, 0)
        let nw_idx = 0;
        let nw_mine = old_values[nw_idx];
        let mut nw_neighbors = [
            old_values[1],                   // 0: east
            old_values[(h - 1) * w],         // 1: north
            old_values[w],                   // 2: south
            old_values[w - 1],               // 3: west
            old_values[(h - 1) * w + 1],     // 4: north-east
            old_values[(h - 1) * w + w - 1], // 5: north-west
            old_values[w + 1],               // 6: south-east
            old_values[w + w - 1],           // 7: south-west
        ];
        if !topology.wrap_x {
            // all western neighbors are the same as mine
            nw_neighbors[3] = nw_mine;
            nw_neighbors[5] = nw_mine;
            nw_neighbors[7] = nw_mine;
        }
        if !topology.wrap_y {
            // all northern neighbors are the same as mine
            nw_neighbors[1] = nw_mine;
            nw_neighbors[5] = nw_mine;
            nw_neighbors[4] = nw_mine;
        }
        f(nw_idx, nw_mine, nw_neighbors);
    }
    {
        // NE corner (w-1, 0)
        let ne_idx = w - 1;
        let ne_mine = old_values[ne_idx];
        let mut ne_neighbors = [
            old_values[0],                   // 0: east
            old_values[(h - 1) * w + w - 1], // 1: north
            old_values[w + w - 1],           // 2: south
            old_values[w - 2],               // 3: west
            old_values[(h - 1) * w],         // 4: north-east
            old_values[(h - 1) * w + w - 2], // 5: north-west
            old_values[w],                   // 6: south-east
            old_values[w + w - 2],           // 7: south-west
        ];
        if !topology.wrap_x {
            // all eastern neighbors are the same as mine
            ne_neighbors[0] = ne_mine;
            ne_neighbors[4] = ne_mine;
            ne_neighbors[6] = ne_mine;
        }
        if !topology.wrap_y {
            // all northern neighbors are the same as mine
            ne_neighbors[1] = ne_mine;
            ne_neighbors[4] = ne_mine;
            ne_neighbors[5] = ne_mine;
        }
        f(ne_idx, ne_mine, ne_neighbors);
    }
    {
        // SW corner (0, h-1)
        let sw_idx = (h - 1) * w;
        let sw_mine = old_values[sw_idx];
        let mut sw_neighbors = [
            old_values[sw_idx + 1],          // 0: east
            old_values[(h - 2) * w],         // 1: north
            old_values[0],                   // 2: south
            old_values[sw_idx + w - 1],      // 3: west
            old_values[(h - 2) * w + 1],     // 4: north-east
            old_values[(h - 2) * w + w - 1], // 5: north-west
            old_values[1],                   // 6: south-east
            old_values[w - 1],               // 7: south-west
        ];
        if !topology.wrap_x {
            // all western neighbors are the same as mine
            sw_neighbors[3] = sw_mine;
            sw_neighbors[5] = sw_mine;
            sw_neighbors[7] = sw_mine;
        }
        if !topology.wrap_y {
            // all southern neighbors are the same as mine
            sw_neighbors[2] = sw_mine;
            sw_neighbors[6] = sw_mine;
            sw_neighbors[7] = sw_mine;
        }
        f(sw_idx, sw_mine, sw_neighbors);
    }
    {
        // SE corner (w-1, h-1)
        let se_idx = h * w - 1;
        let se_mine = old_values[se_idx];
        let mut se_neighbors = [
            old_values[(h - 1) * w],         // 0: east
            old_values[(h - 2) * w + w - 1], // 1: north
            old_values[w - 1],               // 2: south
            old_values[se_idx - 1],          // 3: west
            old_values[(h - 2) * w],         // 4: north-east
            old_values[(h - 2) * w + w - 2], // 5: north-west
            old_values[0],                   // 6: south-east
            old_values[w - 2],               // 7: south-west
        ];
        if !topology.wrap_x {
            // all eastern neighbors are the same as mine
            se_neighbors[0] = se_mine;
            se_neighbors[4] = se_mine;
            se_neighbors[6] = se_mine;
        }
        if !topology.wrap_y {
            // all southern neighbors are the same as mine
            se_neighbors[2] = se_mine;
            se_neighbors[6] = se_mine;
            se_neighbors[7] = se_mine;
        }
        f(se_idx, se_mine, se_neighbors);
    }

    // Handle edges (excluding corners)
    // North edge (excluding corners)
    for i in 1..(w - 1) {
        let idx = i;
        let mine = old_values[idx];
        let mut neighbors = [
            old_values[i + 1],               // 0: east
            old_values[(h - 1) * w + i],     // 1: north
            old_values[w + i],               // 2: south
            old_values[i - 1],               // 3: west
            old_values[(h - 1) * w + i + 1], // 4: north-east
            old_values[(h - 1) * w + i - 1], // 5: north-west
            old_values[w + i + 1],           // 6: south-east
            old_values[w + i - 1],           // 7: south-west
        ];
        if !topology.wrap_y {
            // all northern neighbors are the same as mine
            neighbors[1] = mine;
            neighbors[4] = mine;
            neighbors[5] = mine;
        }
        f(idx, mine, neighbors);
    }

    // South edge (excluding corners)
    for i in 1..(w - 1) {
        let idx = (h - 1) * w + i;
        let mine = old_values[idx];
        let mut neighbors = [
            old_values[idx + 1],             // 0: east
            old_values[(h - 2) * w + i],     // 1: north
            old_values[i],                   // 2: south
            old_values[idx - 1],             // 3: west
            old_values[(h - 2) * w + i + 1], // 4: north-east
            old_values[(h - 2) * w + i - 1], // 5: north-west
            old_values[i + 1],               // 6: south-east
            old_values[i - 1],               // 7: south-west
        ];
        if !topology.wrap_y {
            // all southern neighbors are the same as mine
            neighbors[2] = mine;
            neighbors[6] = mine;
            neighbors[7] = mine;
        }
        f(idx, mine, neighbors);
    }

    // West edge (excluding corners)
    for j in 1..(h - 1) {
        let idx = j * w;
        let mine = old_values[idx];
        let mut neighbors = [
            old_values[idx + 1],             // 0: east
            old_values[(j - 1) * w],         // 1: north
            old_values[(j + 1) * w],         // 2: south
            old_values[idx + w - 1],         // 3: west
            old_values[(j - 1) * w + 1],     // 4: north-east
            old_values[(j - 1) * w + w - 1], // 5: north-west
            old_values[(j + 1) * w + 1],     // 6: south-east
            old_values[(j + 1) * w + w - 1], // 7: south-west
        ];
        if !topology.wrap_x {
            // all western neighbors are the same as mine
            neighbors[3] = mine;
            neighbors[5] = mine;
            neighbors[7] = mine;
        }
        f(idx, mine, neighbors);
    }

    // East edge (excluding corners)
    for j in 1..(h - 1) {
        let idx = j * w + w - 1;
        let mine = old_values[idx];
        let mut neighbors = [
            old_values[j * w],               // 0: east
            old_values[(j - 1) * w + w - 1], // 1: north
            old_values[(j + 1) * w + w - 1], // 2: south
            old_values[idx - 1],             // 3: west
            old_values[(j - 1) * w],         // 4: north-east
            old_values[(j - 1) * w + w - 2], // 5: north-west
            old_values[(j + 1) * w],         // 6: south-east
            old_values[(j + 1) * w + w - 2], // 7: south-west
        ];
        if !topology.wrap_x {
            // all eastern neighbors are the same as mine
            neighbors[0] = mine;
            neighbors[4] = mine;
            neighbors[6] = mine;
        }
        f(idx, mine, neighbors);
    }

    // diffuse inner patches (i.e not on the border)
    for j in 1..(h - 1) {
        for i in 1..(w - 1) {
            let idx = j * w + i;
            let mine = old_values[idx];
            let neighbors = [
                old_values[idx + 1],     // east
                old_values[idx - w],     // north
                old_values[idx + w],     // south
                old_values[idx - 1],     // west
                old_values[idx - w + 1], // north-east
                old_values[idx - w - 1], // north-west
                old_values[idx + w + 1], // south-east
                old_values[idx + w - 1], // south-west
            ];
            f(idx, mine, neighbors);
        }
    }
}

fn idiosyncratic_sum_8(nums: [Float; 8]) -> Float {
    idiosyncratic_sum_4(nums[0..4].try_into().expect("size should be correct"))
        + idiosyncratic_sum_4(nums[4..8].try_into().expect("size should be correct"))
}

/// Sums 4 numbers identically to
/// https://github.com/NetLogo/Tortoise/blob/master/engine/src/main/coffee/engine/core/topology/diffuser.coffee#L253.
/// I'm really not sure why it was done this way.
fn idiosyncratic_sum_4(nums: [Float; 4]) -> Float {
    let (low1, high1) = if nums[0] < nums[1] { (nums[0], nums[1]) } else { (nums[1], nums[0]) };
    let (low2, high2) = if nums[2] < nums[3] { (nums[2], nums[3]) } else { (nums[3], nums[2]) };
    if low2 < high1 && low1 < high2 {
        (low1 + low2) + (high1 + high2)
    } else {
        (low1 + high1) + (low2 + high2)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::sim::topology::TopologySpec;

    /* // Creates a world where the patches have a custom float variable named
    // `my_var`. The parameter`separate_buffer` determines whether this custom
    // variable is stored in a separate buffer from the row base data.
    fn setup_world(
        width: usize,
        height: usize,
        wrap_x: bool,
        wrap_y: bool,
        separate_buffer: bool,
    ) -> (World, AgentFieldDescriptor) {
        let mut world = World::new(TopologySpec {
            min_pxcor: 0,
            max_pycor: 0,
            patches_width: width as i32,
            patches_height: height as i32,
            wrap_x,
            wrap_y,
        });

        let schema = if separate_buffer {
            PatchSchema::new(0, &[(NetlogoInternalType::FLOAT, 1)], &[1])
        } else {
            PatchSchema::new(0, &[(NetlogoInternalType::FLOAT, 0)], &[])
        };

        let var_index = if separate_buffer {
            AgentFieldDescriptor {
                buffer_idx: 1,
                field_idx: 0,
            }
        } else {
            AgentFieldDescriptor {
                buffer_idx: 0,
                field_idx: 1,
            }
        };

        for i in 0..height * width {
            let id = PatchId(i as u32);
            let value = Float::from(9.0);
            let ptr: &mut Float = world
                .patches
                .get_patch_field_mut(id, var_index)
                .expect("patch should exist based on world dimensions")
                .expect_left("patch variable should exist in the buffer as 0.0");
            *ptr = value;
        }

        (world, var_index)
    } */

    /// Tests that [`run_with_neighbors8`] correctly calls the function reports
    /// the values associated with each neighboring patch. `values` is a 1D
    /// array of values associated with each patch.
    /// `expected_reported_neighbors` is the expected values of the neighbors of
    /// each patch, reported in row-major order (not the same as order with
    /// which [`run_with_neighbors8`] reports them; this function handles the
    /// remapping).
    fn test_run_with_neighbors8(
        topology: TopologySpec,
        values: Vec<Float>,
        expected_neighbors_row_major: Vec<[Float; 8]>,
    ) {
        let w = topology.patches_width as usize;
        let h = topology.patches_height as usize;

        assert_eq!(w * h, values.len());
        assert_eq!(w * h, expected_neighbors_row_major.len());

        run_with_neighbors8(&topology, &values, |idx, mine, neighbors| {
            assert_eq!(mine, values[idx]);
            let rm = expected_neighbors_row_major[idx];
            let expected_neighbors = [
                rm[4], // east
                rm[1], // north
                rm[6], // south
                rm[3], // west
                rm[2], // north-east
                rm[0], // north-west
                rm[7], // south-east
                rm[5], // south-west
            ];
            assert_eq!(neighbors, expected_neighbors);
        });
    }

    #[test]
    fn test_run_with_neighbors8_no_wrap() {
        let topology = TopologySpec {
            min_pxcor: 0,
            max_pycor: 0,
            patches_width: 3,
            patches_height: 3,
            wrap_x: false,
            wrap_y: false,
        };

        #[rustfmt::skip]
        let values = Vec::from([
            0, 1, 2,
            3, 4, 5,
            6, 7, 8,
        ].map(Float::from));
        #[rustfmt::skip]
        let expected_neighbors = Vec::from([
            [
                0, 0, 0,
                0,    1,
                0, 3, 4,
            ],
            [
                1, 1, 1,
                0,    2,
                3, 4, 5,
            ],
            [
                2, 2, 2,
                1,    2,
                4, 5, 2,
            ],
            [
                3, 0, 1,
                3,    4,
                3, 6, 7,
            ],
            [
                0, 1, 2,
                3,    5,
                6, 7, 8,
            ],
            [
                1, 2, 5,
                4,    5,
                7, 8, 5,
            ],
            [
                6, 3, 4,
                6,    7,
                6, 6, 6,
            ],
            [
                3, 4, 5,
                6,    8,
                7, 7, 7,
            ],
            [
                4, 5, 8,
                7,    8,
                8, 8, 8,
            ]
        ].map(|row| row.map(Float::from)));

        test_run_with_neighbors8(topology, values, expected_neighbors);
    }

    #[test]
    fn test_run_with_neighbors8_wrap_x() {
        let topology = TopologySpec {
            min_pxcor: 0,
            max_pycor: 0,
            patches_width: 3,
            patches_height: 3,
            wrap_x: true,
            wrap_y: false,
        };

        #[rustfmt::skip]
        let values = Vec::from([
            0, 1, 2,
            3, 4, 5,
            6, 7, 8,
        ].map(Float::from));
        #[rustfmt::skip]
        let expected_neighbors = Vec::from([
            [
                0, 0, 0,
                2,    1,
                5, 3, 4,
            ],
            [
                1, 1, 1,
                0,    2,
                3, 4, 5,
            ],
            [
                2, 2, 2,
                1,    0,
                4, 5, 3,
            ],
            [
                2, 0, 1,
                5,    4,
                8, 6, 7,
            ],
            [
                0, 1, 2,
                3,    5,
                6, 7, 8,
            ],
            [
                1, 2, 0,
                4,    3,
                7, 8, 6,
            ],
            [
                5, 3, 4,
                8,    7,
                6, 6, 6,
            ],
            [
                3, 4, 5,
                6,    8,
                7, 7, 7,
            ],
            [
                4, 5, 3,
                7,    6,
                8, 8, 8,
            ]
        ].map(|row| row.map(Float::from)));

        test_run_with_neighbors8(topology, values, expected_neighbors);
    }

    // TODO tests for wrap y and wrap x and y
}
