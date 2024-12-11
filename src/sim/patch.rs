use std::{
    cell::RefCell,
    ops::{Index, IndexMut},
    rc::Rc,
};

use derive_more::derive::From;

use crate::sim::{
    agent_variables::{CustomAgentVariables, VarIndex, VariableDescriptor, VariableMapper},
    color::Color,
    topology::{CoordInt, PointInt, Topology},
    value::{self, PolyValue},
};

use crate::sim::agent::AgentIndexIntoWorld;

use super::{agent::AgentPosition, topology::Point, world::World};

/// A reference to a patch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From)]
pub struct PatchId {
    // The index of the patch in the [`Patches`] struct.
    grid_index: usize,
}

impl AgentIndexIntoWorld for PatchId {
    type Output<'w> = &'w RefCell<Patch>;

    fn index_into_world<'w>(self, world: &'w World) -> Option<Self::Output<'w>> {
        Some(&world.patches[self])
    }
}

#[derive(Debug)]
pub struct Patches {
    /// The patches in the world, stored in row-major order. The first row
    /// contains the patches with the highest `pycor`, and the first column
    /// contains the patches with the lowest `pxcor`.
    patches: Vec<RefCell<Patch>>,
    /// A mapping between variable names and variable descriptors for patches.
    variable_mapper: VariableMapper<Patch>,
}

impl Patches {
    pub fn new(topology: &Topology) -> Self {
        // create the world
        let mut patches =
            Vec::with_capacity((topology.world_width * topology.world_height) as usize);
        for j in 0..topology.world_height {
            for i in 0..topology.world_width {
                let x = topology.min_pxcor + i as CoordInt;
                let y = topology.max_pycor - j as CoordInt;
                patches.push(RefCell::new(Patch::at(PointInt { x, y })));
            }
        }

        let mut variable_mapper = VariableMapper::new();
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
                .get_mut()
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

    /// # Safety
    ///
    /// The data of the patches must not be borrowed when this function is
    /// called.
    pub fn clear_all_patches(&self) {
        for patch in &self.patches {
            patch.borrow_mut().custom_variables.reset_all();
        }
    }
}

impl Index<PatchId> for Patches {
    type Output = RefCell<Patch>;

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
    pcolor: Color,
    plabel: String, // TODO consider using the netlogo version of string for this
    plabel_color: Color,
    custom_variables: CustomAgentVariables,
    // TODO some way of tracking what turtles are on this patch.
}

impl Patch {
    pub fn at(position: PointInt) -> Self {
        Self {
            position,
            pcolor: Color::BLACK,
            plabel: String::new(),
            plabel_color: Color::BLACK, // TODO use some default label color
            custom_variables: CustomAgentVariables::new(),
        }
    }

    pub fn position_int(&self) -> PointInt {
        self.position
    }

    pub fn get_pcolor(&self) -> Color {
        self.pcolor
    }

    pub fn set_pcolor(&mut self, color: Color) {
        self.pcolor = color;
    }

    pub fn get_plabel(&self) -> &str {
        &self.plabel
    }

    pub fn set_plabel(&mut self, label: String) {
        self.plabel = label;
    }

    pub fn get_plabel_color(&self) -> Color {
        self.plabel_color
    }

    pub fn set_plabel_color(&mut self, color: Color) {
        self.plabel_color = color;
    }

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
