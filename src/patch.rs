use std::{
    ops::{Index, IndexMut},
    rc::Rc,
};

use crate::{
    agent_variables::{CustomAgentVariables, VarIndex, VariableDescriptor, VariableMapper},
    topology::{CoordInt, PointInt, Topology},
    value::{self, Value},
};

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
    /// A mapping between variable names and variable descriptors for patches.
    variable_mapper: VariableMapper<Patch>,
    /// The topology of the world.
    topology: Topology,
}

impl Patches {
    pub fn new(topology: Topology) -> Self {
        // create the world
        let Topology {
            world_width,
            world_height,
            min_pxcor,
            max_pycor,
        } = topology;
        let mut patches = Vec::with_capacity((world_width * world_height) as usize);
        for j in 0..world_height {
            for i in 0..world_width {
                let x = min_pxcor + i as CoordInt;
                let y = max_pycor - j as CoordInt;
                patches.push(Patch::at(PointInt { x, y }));
            }
        }

        let mut variable_mapper = VariableMapper::new();
        let built_in_variables: &[(Rc<str>, fn(&Patch) -> Value)] = &[
            (Rc::from("pxcor"), |patch| {
                value::Float::from(patch.position().x).into()
            }),
            (Rc::from("pycor"), |patch| {
                value::Float::from(patch.position().y).into()
            }),
            // TODO add other variables
        ];
        for (name, getter) in built_in_variables {
            variable_mapper.declare_built_in_variable(name.clone(), *getter);
        }

        Self {
            patches,
            topology,
            variable_mapper,
        }
    }

    pub fn declare_custom_variables(&mut self, variables: Vec<Rc<str>>) {
        let new_to_old_custom_idxs = self.variable_mapper.declare_custom_variables(variables);

        // make sure all patches have the correct mappings in their custom
        // variables
        for patch in &mut self.patches {
            patch
                .custom_variables
                .set_variable_mapping(&new_to_old_custom_idxs);
        }
    }

    pub fn look_up_variable(&self, name: &str) -> Option<VariableDescriptor<Patch>> {
        self.variable_mapper.look_up_variable(name)
    }

    pub fn patch_ids_iter(&self) -> impl Iterator<Item = PatchId> {
        (0..self.patches.len()).map(|i| PatchId { grid_index: i })
    }

    pub fn clear_all_patches(&mut self) {
        for patch in &mut self.patches {
            patch.custom_variables.reset_all();
        }
    }
}

impl Index<PatchId> for Patches {
    type Output = Patch;

    fn index(&self, index: PatchId) -> &Self::Output {
        &self.patches[index.grid_index]
    }
}

impl IndexMut<PatchId> for Patches {
    fn index_mut(&mut self, index: PatchId) -> &mut Self::Output {
        &mut self.patches[index.grid_index]
    }
}

#[derive(Debug)]
pub struct Patch {
    position: PointInt,
    custom_variables: CustomAgentVariables,
    // TODO some way of tracking what turtles are on this patch.
}

impl Patch {
    pub fn at(position: PointInt) -> Self {
        Self {
            position,
            custom_variables: CustomAgentVariables::new(),
        }
    }

    pub fn position(&self) -> PointInt {
        self.position
    }

    pub fn get_custom(&self, var_idx: VarIndex) -> &Value {
        &self.custom_variables[var_idx]
    }

    pub fn set_custom(&mut self, var_idx: VarIndex, value: Value) {
        self.custom_variables[var_idx] = value;
    }
}
