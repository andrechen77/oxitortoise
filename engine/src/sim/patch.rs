use std::{collections::HashMap, mem::offset_of};

use derive_more::derive::From;
use either::Either;

use super::topology::Point;
use crate::{
    mir,
    sim::{
        agent_schema::{AgentFieldDescriptor, AgentSchemaField, PatchSchema},
        color::Color,
        topology::{CoordFloat, PointInt, TopologySpec},
        value::{DynBox, NlFloat},
    },
    util::{
        reflection::{ConcreteTy, Reflect, TypeInfo, TypeInfoOptions},
        row_buffer::{self, RowBuffer, RowSchema},
    },
};

// TODO(doc) the patch id uses -1 as a sentinel value for nobody.
// We must also decide whether the Rust newtype PatchId should allow this
// sentinel value

/// A reference to a patch.
///
/// The patches in the world are indexed in a row-major order, where the first
/// row contains the patches with the highest `pycor`, and the first column
/// contains the patches with the lowest `pxcor`.
///
/// Unlike turtles or links, which only have the fields corresponding to their
/// current breed, patches do not have the concept of breeds so all fields are
/// always active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From)]
#[repr(transparent)]
pub struct PatchId(pub u32);

impl Default for PatchId {
    fn default() -> Self {
        Self(u32::MAX)
    }
}

static PATCH_ID_TYPE_INFO: TypeInfo = TypeInfo::new::<PatchId>(TypeInfoOptions {
    debug_name: "PatchId",
    is_zeroable: false,
    lir_repr: Some(&[lir::ValType::I32]),
});

impl Reflect for PatchId {
    const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&PATCH_ID_TYPE_INFO);
}

pub const OFFSET_PATCHES_TO_DATA: usize = offset_of!(Patches, data);

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
            // TODO(wishlist) we should avoid having to remake the row schemas if we can;
            // we should reuse the ones from the compilation process instead.
            data: patch_schema.make_row_schemas().map(|s| s.map(RowBuffer::new)),
            patch_schema,
            num_patches: topology_spec.num_patches(),
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
                    plabel_color: Color::BLACK, // FIXME use a more sensible default
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
                    if r#type.info().is_zeroable {
                        patches.data[field.buffer_idx as usize]
                            .as_mut()
                            .unwrap()
                            .row_mut(id.0 as usize)
                            .insert_zeroable(field.field_idx as usize);
                    } else {
                        patches.fallback_custom_fields.insert((id, field), DynBox::ZERO);
                    }
                }
                // TODO(wishlist) can reduce code duplication by using a helper
                // function for initialization of the custom fields of turtles,
                // patches, and links
            }
        }

        patches
    }

    pub fn schema(&self) -> &PatchSchema {
        &self.patch_schema
    }

    /// Get a reference to a field of a patch. Returns `None` if the
    /// patch does not exist.
    pub fn get_patch_field<T: Reflect>(
        &self,
        id: PatchId,
        field: AgentFieldDescriptor,
    ) -> Option<Either<&T, &DynBox>> {
        if id.0 >= self.num_patches {
            return None;
        }

        if let Some(field) = self.data[field.buffer_idx as usize]
            .as_ref()
            .expect("data buffer should exist")
            .row(id.0 as usize)
            .get(field.field_idx as usize)
        {
            Some(Either::Left(field))
        } else {
            let fallback = self.fallback_custom_fields.get(&(id, field));
            Some(Either::Right(fallback?))
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
    pub fn get_patch_field_mut<T: Reflect>(
        &mut self,
        id: PatchId,
        field: AgentFieldDescriptor,
    ) -> Option<Either<&mut T, &mut DynBox>> {
        if id.0 >= self.num_patches {
            return None;
        }

        if let Some(field) = self.data[field.buffer_idx as usize]
            .as_mut()
            .expect("data buffer should exist")
            .row_mut(id.0 as usize)
            .get_mut(field.field_idx as usize)
        {
            Some(Either::Left(field))
        } else {
            let fallback = self.fallback_custom_fields.get_mut(&(id, field));
            Some(Either::Right(fallback?))
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

    pub fn set_patch_field<T: Reflect>(
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
    pub fn take_patch_values(&mut self, field: AgentFieldDescriptor) -> row_buffer::Array<NlFloat> {
        // FIXME when taking patch values, we should make sure that patches
        // don't have fallback values for this field
        self.data[field.buffer_idx as usize].as_mut().unwrap().take_array()
    }

    /// # Panics
    ///
    /// Panics if the field cannot be reinterpreted as an array of the specified
    /// type or if it wasn't put into its own field group with no occupancy
    /// bitfield.
    pub fn patch_field_as_mut_array<T: Copy + Reflect>(
        &mut self,
        field: AgentFieldDescriptor,
    ) -> &mut [T] {
        self.data[field.buffer_idx as usize].as_mut().unwrap().as_mut_array()
    }

    /// Resets all patch variables to their default values.
    pub fn clear_patch_variables(&mut self) {
        // TODO(mvp) implement clearing patch variables
    }

    pub fn patch_ids(&self) -> impl Iterator<Item = PatchId> + '_ {
        (0..self.num_patches).map(PatchId)
    }
}

#[derive(Debug)]
pub struct PatchBaseData {
    pub position: Point,
    pub plabel: String, // FIXME consider using the netlogo version of string for this
    pub plabel_color: Color,
    // TODO add some way of tracking what turtles are on this patch.
}

static PATCH_BASE_DATA_TYPE_INFO: TypeInfo = TypeInfo::new::<PatchBaseData>(TypeInfoOptions {
    debug_name: "PatchBaseData",
    is_zeroable: false,
    lir_repr: None,
});

impl Reflect for PatchBaseData {
    const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&PATCH_BASE_DATA_TYPE_INFO);
}

#[derive(Debug, Clone, Copy)]
pub enum PatchVarDesc {
    Pcolor,
    Custom(usize),
}

/// See [`calc_turtle_var_offset`].
pub fn calc_patch_var_offset(mir: &mir::Program, var: PatchVarDesc) -> (usize, usize, usize) {
    fn stride_and_field_offset(
        patch_schema: &PatchSchema,
        field: AgentFieldDescriptor,
    ) -> (usize, usize) {
        // TODO(wishlist) it's inefficient to calculate the schemas every time. see
        // if we can cache this calculation as well as use it for making
        // the workspace
        let schemas: [Option<RowSchema>; 4] = patch_schema.make_row_schemas();
        let row_schema = schemas[usize::from(field.buffer_idx)].as_ref().unwrap();
        let field_offset = row_schema.field(usize::from(field.field_idx)).offset;
        let stride = row_schema.stride();
        (stride, field_offset)
    }

    let patch_schema = mir.patch_schema.as_ref().unwrap();
    let (buffer_idx, stride, field_offset) = match var {
        PatchVarDesc::Custom(field_id) => {
            let field_desc = patch_schema.custom_fields()[field_id];
            let (stride, field_offset) = stride_and_field_offset(patch_schema, field_desc);
            (field_desc.buffer_idx, stride, field_offset)
        }
        PatchVarDesc::Pcolor => {
            let base_data_desc = patch_schema.base_data();
            let (stride, field_offset) = stride_and_field_offset(patch_schema, base_data_desc);
            (base_data_desc.buffer_idx, stride, field_offset)
        }
    };
    let buffer_offset =
        offset_of!(Patches, data) + (usize::from(buffer_idx) * size_of::<Option<RowBuffer>>());
    (buffer_offset, stride, field_offset)
}

pub fn patch_var_type(schema: &PatchSchema, var: PatchVarDesc) -> ConcreteTy {
    match var {
        PatchVarDesc::Pcolor => Color::CONCRETE_TY,
        PatchVarDesc::Custom(field) => {
            let AgentSchemaField::Other(ty) = schema[schema.custom_fields()[field]] else {
                unreachable!("this is a custom field, so it cannot be part of the base data");
            };
            ty
        }
    }
}
