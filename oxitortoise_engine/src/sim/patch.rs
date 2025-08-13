use std::{collections::HashMap, mem::offset_of};

use derive_more::derive::From;
use either::Either;

use super::topology::Point;
use crate::{
    sim::{
        agent_schema::{AgentFieldDescriptor, AgentSchemaField, PatchSchema},
        color::Color,
        topology::{CoordFloat, PointInt, TopologySpec},
        value::{DynBox, Float},
    },
    util::row_buffer::{self, RowBuffer},
};

// TODO make documentation better
/// The patches in the world are indexed in a row-major order, where the first
/// row contains the patches with the highest `pycor`, and the first column
/// contains the patches with the lowest `pxcor`.
///
/// Unlike turtles or links, which only have the fields corresponding to their
/// current breed, patches do not have the concept of breeds so all fields are
/// always active.

// TODO document that the Patch Id uses -1 as a sentinel value for nobody
/// A reference to a patch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From)]
#[repr(transparent)]
pub struct PatchId(pub u32);

impl Default for PatchId {
    fn default() -> Self {
        Self(u32::MAX)
    }
}

#[no_mangle]
static OFFSET_PATCHES_TO_DATA: usize = offset_of!(Patches, data);

#[derive(Debug)]
pub struct Patches {
    /// The buffers that store the data for the patches. Each patch is
    /// represented by a row in all buffers. There are multiple buffers to
    /// allow for SoA-style data locality of certain fields.
    data: [Option<RowBuffer>; 4],
    /// The fields of a patch.
    patch_schema: PatchSchema,
    /// The number of patches in the world.
    num_patches: u32,
    /// Fallback storage for custom fields whose type doesn't match the
    /// compile-time type.
    fallback_custom_fields: HashMap<(PatchId, AgentFieldDescriptor), DynBox>,
}

impl Patches {
    pub fn new(patch_schema: PatchSchema, topology_spec: &TopologySpec) -> Self {
        let mut patches = Self {
            data: patch_schema.make_row_buffers(),
            patch_schema,
            num_patches: topology_spec.num_patches() as u32,
            fallback_custom_fields: HashMap::new(),
        };

        // populate the patches
        for buffer in patches.data.iter_mut().filter_map(|b| b.as_mut()) {
            buffer.ensure_capacity(patches.num_patches as usize);
        }
        let TopologySpec { min_pxcor, max_pycor, patches_height, patches_width, .. } =
            topology_spec;
        for j in 0..*patches_height {
            for i in 0..*patches_width {
                let x = min_pxcor + i;
                let y = max_pycor - j;
                let position = Point { x: x as CoordFloat, y: y as CoordFloat };
                // topology_spec.patch_at(position) should just return an
                // increasing index anyway but this is more robust (even though
                // it's literally the same thing just requiring more
                // optimization)
                let id = topology_spec.patch_at(PointInt { x, y });

                // initialize base data
                let base_data = PatchBaseData {
                    position,
                    plabel: String::new(),
                    plabel_color: Color::BLACK, // TODO use a more sensible default
                };
                patches.data[0].as_mut().unwrap().row_mut(id.0 as usize).insert(0, base_data);

                // initialize other builtins
                let pcolor_desc = patches.patch_schema.pcolor();
                patches.data[pcolor_desc.buffer_idx as usize]
                    .as_mut()
                    .unwrap()
                    .row_mut(id.0 as usize)
                    .insert_zeroable(pcolor_desc.field_idx as usize);

                // initialize custom fields
                for &field in patches.patch_schema.custom_fields() {
                    let AgentSchemaField::Other(r#type) = &patches.patch_schema[field] else {
                        panic!("field at index {:?} should be a custom field", field);
                    };
                    if r#type.is_numeric_zeroable() {
                        patches.data[field.buffer_idx as usize]
                            .as_mut()
                            .unwrap()
                            .row_mut(id.0 as usize)
                            .insert_zeroable(field.field_idx as usize);
                    } else {
                        patches.fallback_custom_fields.insert((id, field), DynBox::ZERO);
                    }
                }
                // TODO can reduce code duplication by using a helper function
                // for initialization of the custom fields of turtles, patches,
                // and links
            }
        }

        patches
    }

    pub fn schema(&self) -> &PatchSchema {
        &self.patch_schema
    }

    /// Get a reference to a field of a patch. Returns `None` if the
    /// patch does not exist.
    pub fn get_patch_field<T: 'static>(
        &self,
        id: PatchId,
        field: AgentFieldDescriptor,
    ) -> Option<Either<&T, &DynBox>> {
        if id.0 >= self.num_patches {
            return None;
        }

        if let Some(field) = self.data[field.buffer_idx as usize]
            .as_ref()
            .unwrap()
            .row(id.0 as usize)
            .get(field.field_idx as usize)
        {
            Some(Either::Left(field))
        } else {
            let fallback = self.fallback_custom_fields.get(&(id, field));
            Some(Either::Right(fallback.unwrap())) // TODO handle unwrap
        }
    }

    pub fn get_patch_base_data(&self, id: PatchId) -> Option<&PatchBaseData> {
        self.get_patch_field(id, AgentFieldDescriptor::BASE_DATA)
            .map(|either| either.expect_left("base data should always exist in the row buffer"))
    }

    pub fn get_patch_pcolor(&self, id: PatchId) -> Option<&Color> {
        self.get_patch_field(id, self.patch_schema.pcolor())
            .map(|either| either.expect_left("pcolor should always exist in the row buffer"))
    }

    /// Get a mutable reference to a field of a patch. Returns `None` if the
    /// patch does not exist.
    pub fn get_patch_field_mut<T: 'static>(
        &mut self,
        id: PatchId,
        field: AgentFieldDescriptor,
    ) -> Option<Either<&mut T, &mut DynBox>> {
        if id.0 >= self.num_patches {
            return None;
        }

        if let Some(field) = self.data[field.buffer_idx as usize]
            .as_mut()
            .unwrap()
            .row_mut(id.0 as usize)
            .get_mut(field.field_idx as usize)
        {
            Some(Either::Left(field))
        } else {
            let fallback = self.fallback_custom_fields.get_mut(&(id, field));
            Some(Either::Right(fallback.unwrap())) // TODO handle unwrap
        }
    }

    pub fn get_patch_base_data_mut(&mut self, id: PatchId) -> Option<&mut PatchBaseData> {
        self.get_patch_field_mut(id, AgentFieldDescriptor::BASE_DATA)
            .map(|either| either.expect_left("base data should always exist in the row buffer"))
    }

    pub fn get_patch_pcolor_mut(&mut self, id: PatchId) -> Option<&mut Color> {
        self.get_patch_field_mut(id, self.patch_schema.pcolor())
            .map(|either| either.expect_left("pcolor should always exist in the row buffer"))
    }

    pub fn set_patch_field<T: 'static>(
        &mut self,
        id: PatchId,
        field: AgentFieldDescriptor,
        value: T,
    ) {
        if id.0 >= self.num_patches {
            panic!("patch does not exist");
        }
        self.data[field.buffer_idx as usize]
            .as_mut()
            .unwrap()
            .row_mut(id.0 as usize)
            .insert(field.field_idx as usize, value);
        if self.fallback_custom_fields.contains_key(&(id, field)) {
            self.fallback_custom_fields.remove(&(id, field));
        }
    }

    /// # Panics
    ///
    /// Panics if the field cannot be interpreted as a float or if it wasn't
    /// put into its own field group with no occupancy bitfield.
    pub fn take_patch_values(&mut self, field: AgentFieldDescriptor) -> row_buffer::Array<Float> {
        // TODO make sure that patches don't have a fallback value for this
        // field
        self.data[field.buffer_idx as usize].as_mut().unwrap().take_array()
    }

    /// # Panics
    ///
    /// Panics if the field cannot be reinterpreted as an array of the specified
    /// type or if it wasn't put into its own field group with no occupancy
    /// bitfield.
    pub fn patch_field_as_mut_array<T: Copy + 'static>(
        &mut self,
        field: AgentFieldDescriptor,
    ) -> &mut [T] {
        self.data[field.buffer_idx as usize].as_mut().unwrap().as_mut_array()
    }

    /// Resets all patch variables to their default values.
    pub fn clear_patch_variables(&mut self) {
        // TODO implement this
    }

    pub fn patch_ids(&self) -> impl Iterator<Item = PatchId> + '_ {
        (0..self.num_patches).map(PatchId)
    }
}

#[derive(Debug)]
pub struct PatchBaseData {
    pub position: Point,
    pub plabel: String, // TODO consider using the netlogo version of string for this
    pub plabel_color: Color,
    // TODO some way of tracking what turtles are on this patch.
}
