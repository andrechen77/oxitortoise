use std::{
    cell::RefCell,
    ops::{Index, IndexMut},
    rc::Rc,
};

use derive_more::derive::From;

use crate::sim::{
    agent_variables::{CustomAgentVariables, VarIndex, VariableDescriptor, VariableMapper},
    color::Color,
    topology::{CoordInt, PointInt, TopologySpec},
    value::{self, PolyValue},
};

use crate::sim::agent::AgentIndexIntoWorld;

use super::{agent::AgentPosition, topology::Point, world::World};

/// A reference to a patch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From)]
pub struct PatchId {
    // The index of the patch in the [`Patches`] struct.
    pub grid_index: usize,
}

impl AgentIndexIntoWorld for PatchId {
    type Output<'w> = &'w Patch;

    fn index_into_world(self, world: &World) -> Option<Self::Output<'_>> {
        Some(&world.patches[self])
    }
}

#[derive(Debug)]
pub struct Patches {
    /// The patches in the world, stored in row-major order. The first row
    /// contains the patches with the highest `pycor`, and the first column
    /// contains the patches with the lowest `pxcor`.
    patches: Vec<Patch>,
    /// A mapping between variable names and variable descriptors for patches.
    variable_mapper: VariableMapper<Patch>,
}

impl Patches {
    pub fn new(topology_spec: &TopologySpec) -> Self {
        let TopologySpec {
            min_pxcor,
            max_pycor,
            patches_height,
            patches_width,
            ..
        } = topology_spec;

        // create the world
        let mut patches = Vec::with_capacity((patches_width * patches_height) as usize);
        for j in 0..*patches_height {
            for i in 0..*patches_width {
                let x = min_pxcor + i as CoordInt;
                let y = max_pycor - j as CoordInt;
                patches.push(Patch::at(
                    PatchId {
                        grid_index: patches.len(),
                    },
                    PointInt { x, y },
                ));
            }
        }

        let mut variable_mapper = VariableMapper::new();
        #[allow(clippy::type_complexity)]
        let built_in_variables: &[(Rc<str>, fn(&Patch) -> PolyValue)] = &[
            (Rc::from("pxcor"), |patch| {
                value::Float::from(patch.position_int().x).into()
            }),
            (Rc::from("pycor"), |patch| {
                value::Float::from(patch.position_int().y).into()
            }),
            // TODO add other variables
        ];
        for (name, getter) in built_in_variables {
            variable_mapper.declare_built_in_variable(name.clone(), *getter);
        }

        Self {
            patches,
            variable_mapper,
        }
    }

    pub fn declare_custom_variables(&mut self, variables: Vec<Rc<str>>) {
        let new_to_old_custom_idxs = self.variable_mapper.declare_custom_variables(variables);

        // make sure all patches have the correct mappings in their custom
        // variables
        for patch in &mut self.patches {
            patch
                .data
                .get_mut()
                .custom_variables
                .set_variable_mapping(&new_to_old_custom_idxs);
        }
    }

    pub fn look_up_variable(&self, name: &str) -> Option<VariableDescriptor<Patch>> {
        self.variable_mapper.look_up_variable(name)
    }

    pub fn patches_iter(&self) -> impl Iterator<Item = &Patch> {
        self.patches.iter()
    }

    pub fn patch_ids_iter(&self) -> impl Iterator<Item = PatchId> {
        (0..self.patches.len()).map(|i| PatchId { grid_index: i })
    }

    /// # Safety
    ///
    /// The data of the patches must not be borrowed when this function is
    /// called.
    pub fn clear_all_patches(&self) {
        for patch in &self.patches {
            patch.data.borrow_mut().custom_variables.reset_all();
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
    id: PatchId,
    position: PointInt,
    pub data: RefCell<PatchData>,
}

#[derive(Debug, Default)]
pub struct PatchData {
    pub pcolor: Color,
    pub plabel: String, // TODO consider using the netlogo version of string for this
    pub plabel_color: Color,
    custom_variables: CustomAgentVariables,
    // TODO some way of tracking what turtles are on this patch.
}

impl Patch {
    pub fn at(id: PatchId, position: PointInt) -> Self {
        Self {
            id,
            position,
            data: Default::default(),
        }
    }

    pub fn id(&self) -> PatchId {
        self.id
    }

    pub fn position_int(&self) -> PointInt {
        self.position
    }
}

impl PatchData {
    pub fn get_custom(&self, var_idx: VarIndex) -> &PolyValue {
        &self.custom_variables[var_idx]
    }

    pub fn set_custom(&mut self, var_idx: VarIndex, value: PolyValue) {
        self.custom_variables[var_idx] = value;
    }
}

impl AgentPosition for Patch {
    fn position(&self) -> Point {
        self.position.into()
    }
}
