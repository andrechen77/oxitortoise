use core::panic;
use std::ops::{Index, IndexMut};

use derive_more::derive::{From, TryFrom};

use crate::{
    sim::{
        color::Color,
        topology::{CoordInt, TopologySpec},
    },
    util::cell::RefCell,
};

use crate::sim::agent::AgentIndexIntoWorld;

use super::{agent::AgentPosition, topology::Point, value::{Float, String, TryAsFloat}, variables::{DynTypedVec, InUntypedVal}, world::World};

/// A reference to a patch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From)]
#[repr(transparent)]
pub struct PatchId(pub usize);

impl TryAsFloat for PatchId {
    fn try_as_float(&self) -> Option<Float> {
        None
    }
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
    // /// A mapping between variable names and variable descriptors for patches.
    // variable_mapper: VariableMapper,
    // TODO add back variable mapper
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
                let x = (min_pxcor + i as CoordInt).into();
                let y = (max_pycor - j as CoordInt).into();
                patches.push(Patch::at(PatchId(patches.len()), Point { x, y }));
            }
        }

        // let mut variable_mapper = VariableMapper::new();
        // #[allow(clippy::type_complexity)]
        // let built_in_variables: &[Rc<str>] = &[
        //     Rc::from("pxcor"),
        //     Rc::from("pycor"),
        //     // TODO add other variables
        // ];
        // for name in built_in_variables {
        //     variable_mapper.declare_built_in_variable(name.clone());
        // }

        Self {
            patches,
            // variable_mapper,
        }
    }

    // TODO reimplement functionality for declaring custom variables
    // pub fn declare_custom_variables(&mut self, variables: Vec<Rc<str>>) {
    //     let new_to_old_custom_idxs = self.variable_mapper.declare_custom_variables(variables);

    //     // make sure all patches have the correct mappings in their custom
    //     // variables
    //     for patch in &mut self.patches {
    //         patch
    //             .data
    //             .get_mut()
    //             .custom_variables
    //             .set_variable_mapping(&new_to_old_custom_idxs);
    //     }
    // }

    // pub fn look_up_variable(&self, name: &str) -> Option<VariableDescriptor> {
    //     self.variable_mapper.look_up_variable(name)
    // }

    pub fn patches_iter(&self) -> impl Iterator<Item = &Patch> {
        self.patches.iter()
    }

    pub fn patch_ids_iter(&self) -> impl Iterator<Item = PatchId> {
        (0..self.patches.len()).map(PatchId)
    }

    /// # Safety
    ///
    /// The data of the patches must not be borrowed when this function is
    /// called.
    pub fn clear_all_patches(&self) {
        for patch in &self.patches {
            // TODO reset other patch variables
            patch.data.borrow_mut().custom_variables.reset();
        }
    }
}

impl Index<PatchId> for Patches {
    type Output = Patch;

    fn index(&self, index: PatchId) -> &Self::Output {
        &self.patches[index.0]
    }
}

impl IndexMut<PatchId> for Patches {
    fn index_mut(&mut self, index: PatchId) -> &mut Self::Output {
        &mut self.patches[index.0]
    }
}

#[derive(Debug)]
pub struct Patch {
    id: PatchId,
    position: Point,
    pub data: RefCell<PatchData>,
}

#[derive(Debug, Default)]
pub struct PatchData {
    pub pcolor: Color,
    pub plabel: String,
    pub plabel_color: Color,
    custom_variables: DynTypedVec<6>, // TODO pick a better number than 6
    // TODO some way of tracking what turtles are on this patch.
}

impl Patch {
    pub fn at(id: PatchId, position: Point) -> Self {
        Self {
            id,
            position,
            data: Default::default(),
        }
    }

    pub fn id(&self) -> PatchId {
        self.id
    }

    // TODO replace this with a proper Index impl
    // TODO add an unsafe version that assumes that the value is a Float
    pub fn get_numeric(&self, var: PatchVarDescriptor) -> Float {
        let magic = var.0;
        if magic < BUILTIN_RESERVED {
            let data = &self.data.borrow().custom_variables[magic as usize];
            if let Some(v) = data.try_as_float() {
                v
            } else {
                todo!("not a numeric descriptor")
            }
        } else if let Ok(magic) = magic.try_into() {
            match magic {
                BuiltInDescMagicNumbers::Pxcor => self.position.x,
                BuiltInDescMagicNumbers::Pycor => self.position.y,
                BuiltInDescMagicNumbers::Pcolor => self.data.borrow().pcolor.to_float(),
                BuiltInDescMagicNumbers::Plabel => todo!(),
                BuiltInDescMagicNumbers::PlabelColor => self.data.borrow().plabel_color.to_float(),
            }
        } else {
            todo!("not a valid descriptor")
        }
    }

    // TODO replace with a proper IndexMut impl
    // pub fn set_numeric(&self, var: PatchVarDescriptor, value: Float) {
    //     let magic = var.0;
    //     if magic < BUILTIN_RESERVED {
    //         let mut data = self.data.borrow_mut();
    //         data.custom_variables[magic as usize] = value.into();
    //     } else if let Ok(magic) = magic.try_into() {
    //         match magic {
    //             BuiltInDescMagicNumbers::Pxcor => self.position.x = value,
    //             BuiltInDescMagicNumbers::Pycor => self.position.y = value,
    //             BuiltInDescMagicNumbers::Pcolor => self.data.borrow_mut().pcolor = Color::from_float(value),
    //             BuiltInDescMagicNumbers::Plabel => todo!(),
    //             BuiltInDescMagicNumbers::PlabelColor => self.data.borrow_mut().plabel_color = Color::from_float(value),
    //         }
    //     } else {
    //         todo!("not a valid descriptor")
    //     }
    // }
}

impl PatchData {
    pub fn get_mut<T: InUntypedVal>(&mut self, var: PatchVarDescriptor) -> Option<&mut T> {
        let magic = var.0;
        if magic < BUILTIN_RESERVED {
            self.custom_variables.get_mut(magic as usize)
        } else if let Ok(magic) = magic.try_into() {
            match magic {
                BuiltInDescMagicNumbers::Pxcor => panic!("cannot mutate pxcor"),
                BuiltInDescMagicNumbers::Pycor => panic!("cannot mutate pycor"),
                BuiltInDescMagicNumbers::Pcolor => self.pcolor.try_as_float().map(|_| &mut self.pcolor),
                BuiltInDescMagicNumbers::Plabel => todo!(),
                BuiltInDescMagicNumbers::PlabelColor => todo!(),
            }
        } else {
            todo!("not a valid descriptor")
        }
    }
}

impl AgentPosition for Patch {
    fn position(&self) -> Point {
        self.position.into()
    }
}

/// Describes the location of a certain variable in a patch.
#[repr(transparent)]
pub struct PatchVarDescriptor(u8);

/// All magic numbers i below this value describe the custom variable at
/// index i. Numbers at or above this value are reserved for built-in
/// variables.
const BUILTIN_RESERVED: u8 = 128;

/// Magic numbers for [`PatchVarDescriptor`] describing built-in variables.
#[derive(TryFrom)]
#[try_from(repr)]
#[repr(u8)]
enum BuiltInDescMagicNumbers {
    Pxcor = BUILTIN_RESERVED,
    Pycor,
    Pcolor,
    Plabel,
    PlabelColor,
}
