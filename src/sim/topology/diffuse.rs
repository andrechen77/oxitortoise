use crate::sim::{agent_variables::VarIndex, patch::PatchId, value::Float, world::World};

pub fn diffuse_8(world: &World, patch_variable: VarIndex, fraction: Float) {
    // create a scratch array with all the current variable values
    let prev = create_prev_values_array_with_ghost_patches(world, patch_variable);

    let top = world.topology.spec();
    let w = top.patches_width as usize + 2;
    let h = top.patches_height as usize + 2;
    debug_assert!(prev.len() == w * h);

    // diffuse into all patches. j and i are indices into the prev array
    for j in 1..(h - 1) {
        for i in 1..(w - 1) {
            let c = prev[j * w + i];
            let nn = prev[(j - 1) * w + i];
            let ss = prev[(j + 1) * w + i];
            let ee = prev[j * w + i + 1];
            let ww = prev[j * w + i - 1];
            let nw = prev[(j - 1) * w + i - 1];
            let ne = prev[(j - 1) * w + i + 1];
            let se = prev[(j + 1) * w + i + 1];
            let sw = prev[(j + 1) * w + i - 1];

            // the order in which we sum them is important, apparently
            // https://github.com/NetLogo/Tortoise/blob/master/engine/src/main/coffee/engine/core/topology/diffuser.coffee#L8
            let neighbors_sum = idiosyncratic_sum_8([ee, nn, ss, ww, ne, nw, se, sw]);
            let new_val = c + fraction * (neighbors_sum / Float::new(8.0) - c);

            // convert from prev array indexing to the actual patch array indexing
            let id = PatchId((j - 1) * top.patches_width as usize + (i - 1));

            // set the value
            world.patches[id]
                .data
                .borrow_mut()
                .set_custom(patch_variable, new_val.into());
        }
    }
}

/// Creates an array holding the previous values of the specified variable in
/// all the patches, stored in row-major order. To facilitate either wrapping
/// behavior or boundary behavior during diffusion, this array stores variables
/// one past each world boundary. Thus, if the world is 10 patches by 10
/// patches, this array will store 12 * 12 = 144 values. The values of these
/// extra array elements ("ghost patches") are either the variable values of the
/// patch across the world (for wrapping behavior) or the value of the patch at
/// the edge (for boundary behavior).
fn create_prev_values_array_with_ghost_patches(world: &World, var: VarIndex) -> Vec<Float> {
    fn patch_value(world: &World, id: PatchId, var: VarIndex) -> Float {
        *world.patches[id]
            .data
            .borrow()
            .get_custom(var)
            .get::<Float>()
            .expect("expected a float")
    }

    let top = &world.topology;
    let width = top.patches_width() as usize;
    let height = top.patches_height() as usize;

    let mut values = Vec::with_capacity((height + 2) * (width + 2));

    // TODO all these shenanigans of if statements could be made more elegant by
    // creating the array assuming boundary behavior first, and then swapping
    // the values of the ghost patches at the edges if wrapping behavior is
    // enabled

    // fill in the first row of ghost patches
    if top.wrap_y() {
        // handle the top-left ghost corner
        let id = if top.wrap_x() {
            PatchId((height - 1) * width + (width - 1)) // bottom-right
        } else {
            PatchId(0) // top-left
        };
        values.push(patch_value(world, id, var));

        // handle the top-i ghost edges
        for i in 0..width {
            let id = PatchId((height - 1) * width + i); // bottom-i
            values.push(patch_value(world, id, var));
        }

        // handle the top-right ghost corner
        let id = if top.wrap_x() {
            PatchId((height - 1) * width) // bottom-left
        } else {
            PatchId(width - 1) // top-right
        };
        values.push(patch_value(world, id, var));
    } else {
        // handle the top-left ghost corner
        let id = PatchId(0); // top-left
        values.push(patch_value(world, id, var));

        // handle the top-i ghost edges
        for i in 0..width {
            let id = PatchId(i); // top-i
            values.push(patch_value(world, id, var));
        }

        // handle the top-right ghost corner
        let id = PatchId(width - 1); // top-right
        values.push(patch_value(world, id, var));
    }
    debug_assert!(values.len() == width + 2);

    // fill in the middle rows
    for j in 0..height {
        // handle the left ghost edge
        let id = if top.wrap_x() {
            PatchId(j * width + (width - 1)) // j-right
        } else {
            PatchId(j * width) // j-left
        };
        values.push(patch_value(world, id, var));

        // handle the center
        for i in 0..width {
            let id = PatchId(j * width + i);
            values.push(patch_value(world, id, var));
        }

        // handle the right ghost edge
        let id = if top.wrap_x() {
            PatchId(j * width) // j-left
        } else {
            PatchId(j * width + (width - 1)) // j-right
        };
        values.push(patch_value(world, id, var));

        debug_assert!(values.len() == (j + 2) * (width + 2));
    }

    // fill in the last row of ghost patches
    if top.wrap_y() {
        // handle the bottom-left ghost corner
        let id = if top.wrap_x() {
            PatchId(width - 1) // top-right
        } else {
            PatchId((height - 1) * width) // bottom-left
        };
        values.push(patch_value(world, id, var));

        // handle the bottom-i ghost edges
        for i in 0..width {
            let id = PatchId(i); // top-i
            values.push(patch_value(world, id, var));
        }

        // handle the bottom-right ghost corner
        let id = if top.wrap_x() {
            PatchId(0) // top-left
        } else {
            PatchId((height - 1) * width + (width - 1)) // bottom-right
        };
        values.push(patch_value(world, id, var));
    } else {
        // handle the bottom-left ghost corner
        let id = PatchId((height - 1) * width); // bottom-left
        values.push(patch_value(world, id, var));

        // handle the bottom-i ghost edges
        for i in 0..width {
            let id = PatchId((height - 1) * width + i); // bottom-i
            values.push(patch_value(world, id, var));
        }

        // handle the bottom-right ghost corner
        let id = PatchId((height - 1) * width + (width - 1)); // bottom-right
        values.push(patch_value(world, id, var));
    }

    values
}

fn idiosyncratic_sum_8(nums: [Float; 8]) -> Float {
    idiosyncratic_sum_4(nums[0..4].try_into().expect("size should be correct"))
        + idiosyncratic_sum_4(nums[4..8].try_into().expect("size should be correct"))
}

/// Sums 4 numbers identically to
/// https://github.com/NetLogo/Tortoise/blob/master/engine/src/main/coffee/engine/core/topology/diffuser.coffee#L253.
/// I'm really not sure why it was done this way.
fn idiosyncratic_sum_4(nums: [Float; 4]) -> Float {
    let (low1, high1) = if nums[0] < nums[1] {
        (nums[0], nums[1])
    } else {
        (nums[1], nums[0])
    };
    let (low2, high2) = if nums[2] < nums[3] {
        (nums[2], nums[3])
    } else {
        (nums[3], nums[2])
    };
    if low2 < high1 && low1 < high2 {
        (low1 + low2) + (high1 + high2)
    } else {
        (low1 + high1) + (low2 + high2)
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use crate::sim::topology::TopologySpec;

    use super::*;

    fn setup_world(width: usize, height: usize, wrap_x: bool, wrap_y: bool) -> World {
        let mut world = World::new(TopologySpec {
            min_pxcor: 0,
            max_pycor: 0,
            patches_width: width as i64,
            patches_height: height as i64,
            wrap_x,
            wrap_y,
        });

        world
            .patches
            .declare_custom_variables(vec![Rc::from("my_var")]);

        let var_index = VarIndex::from_index(0);
        for j in 0..height {
            for i in 0..width {
                let id = PatchId(j * width + i);
                world.patches[id]
                    .data
                    .borrow_mut()
                    .set_custom(var_index, Float::from((i + j * width) as f64).into());
            }
        }

        world
    }

    #[test]
    fn test_create_prev_values_array_no_wrapping() {
        let width = 3;
        let height = 3;

        let world = setup_world(width, height, false, false);

        let var_index = VarIndex::from_index(0);
        let prev_values = create_prev_values_array_with_ghost_patches(&world, var_index);
        #[rustfmt::skip]
        let expected_values: Vec<Float> = vec![
            0, 0, 1, 2, 2,
            0, 0, 1, 2, 2,
            3, 3, 4, 5, 5,
            6, 6, 7, 8, 8,
            6, 6, 7, 8, 8,
        ]
        .into_iter()
        .map(|f| Float::from(f))
        .collect();

        assert_eq!(prev_values, expected_values);
    }

    #[test]
    fn test_create_prev_values_array_wrap_x_only() {
        let width = 3;
        let height = 3;

        let world = setup_world(width, height, true, false);

        let var_index = VarIndex::from_index(0);
        let prev_values = create_prev_values_array_with_ghost_patches(&world, var_index);
        #[rustfmt::skip]
        let expected_values: Vec<Float> = vec![
            0, 0, 1, 2, 2,
            2, 0, 1, 2, 0,
            5, 3, 4, 5, 3,
            8, 6, 7, 8, 6,
            6, 6, 7, 8, 8,
        ]
        .into_iter()
        .map(|f| Float::from(f))
        .collect();

        assert_eq!(prev_values, expected_values);
    }

    #[test]
    fn test_create_prev_values_array_wrap_y_only() {
        let width = 3;
        let height = 3;

        let world = setup_world(width, height, false, true);

        let var_index = VarIndex::from_index(0);
        let prev_values = create_prev_values_array_with_ghost_patches(&world, var_index);
        #[rustfmt::skip]
        let expected_values: Vec<Float> = vec![
            0, 6, 7, 8, 2,
            0, 0, 1, 2, 2,
            3, 3, 4, 5, 5,
            6, 6, 7, 8, 8,
            6, 0, 1, 2, 8,
        ]
        .into_iter()
        .map(|f| Float::from(f))
        .collect();

        assert_eq!(prev_values, expected_values);
    }

    #[test]
    fn test_create_prev_values_array_wrap_both() {
        let width = 3;
        let height = 3;

        let world = setup_world(width, height, true, true);

        let var_index = VarIndex::from_index(0);
        let prev_values = create_prev_values_array_with_ghost_patches(&world, var_index);
        #[rustfmt::skip]
        let expected_values: Vec<Float> = vec![
            8, 6, 7, 8, 6,
            2, 0, 1, 2, 0,
            5, 3, 4, 5, 3,
            8, 6, 7, 8, 6,
            2, 0, 1, 2, 0,
        ]
        .into_iter()
        .map(|f| Float::from(f))
        .collect();

        assert_eq!(prev_values, expected_values);
    }
}
