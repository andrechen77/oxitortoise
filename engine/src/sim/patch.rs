use std::{
    collections::HashMap,
    fmt::{self, Write},
    mem::offset_of,
    ops::Index,
    rc::Rc,
    sync::Arc,
};

use derive_more::derive::From;
use either::Either;
use pretty_print::PrettyPrinter;

use super::topology::Point;
use crate::{
    mir,
    sim::{
        agent_schema::{AgentFieldDescriptor, AgentSchemaField, AgentSchemaFieldGroup},
        color::Color,
        topology::{CoordFloat, PointInt, TopologySpec},
        value::{NlBool, NlFloat, NlList, NlString, PackedAny},
    },
    util::{
        reflection::{ConcreteTy, ConstTypeName, Reflect, TypeInfo, TypeInfoOptions},
        row_buffer::{self, RowBuffer, RowSchema},
    },
};

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
    is_zeroable: false,
    mem_repr: Some(&[(0, lir::MemOpType::I32)]),
});

impl ConstTypeName for PatchId {
    const TYPE_NAME: &'static str = "PatchId";
}

unsafe impl Reflect for PatchId {
    const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&PATCH_ID_TYPE_INFO);
}

/// Exactly the same as [`PatchId`], but it can represent "nobody" at the -1
/// value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From)]
#[repr(transparent)]
pub struct OptionPatchId(pub u32);

// make a copy with a different identity
static OPTION_PATCH_ID_TYPE_INFO: TypeInfo = PATCH_ID_TYPE_INFO;
unsafe impl Reflect for OptionPatchId {
    const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&OPTION_PATCH_ID_TYPE_INFO);
}

impl OptionPatchId {
    pub const NOBODY: Self = Self(u32::MAX);
}

impl From<PatchId> for OptionPatchId {
    fn from(value: PatchId) -> Self {
        OptionPatchId(value.0)
    }
}

pub struct Patches {
    /// The buffers that store the data for the patches. Each patch is
    /// represented by a row in all buffers. There are multiple buffers to
    /// allow for SoA-style data locality of certain fields.
    pub data: [Option<RowBuffer>; 4],
    /// The fields of a patch.
    patch_schema: PatchSchema,
    /// The number of patches in the world.
    num_patches: u32,
    /// Fallback storage for custom fields whose type doesn't match the
    /// compile-time type.
    fallback_custom_fields: HashMap<(PatchId, AgentFieldDescriptor), PackedAny>,
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
                for &(_, field) in patches.patch_schema.custom_fields() {
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
                        patches.fallback_custom_fields.insert((id, field), PackedAny::ZERO);
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
    ) -> Option<Either<&T, &PackedAny>> {
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
    ) -> Option<Either<&mut T, &mut PackedAny>> {
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

impl fmt::Debug for Patches {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut p = PrettyPrinter::new(f);
        let Patches { data: _, patch_schema, num_patches, fallback_custom_fields: _ } = self;
        p.add_struct("Patches", |p| {
            p.add_field_with("patch_schema", |p| write!(p, "{:?}", patch_schema))?;
            p.add_field_with("num_patches", |p| write!(p, "{}", num_patches))?;
            p.add_field_with("patches", |p| {
                p.add_map(
                    self.patch_ids().map(|id| (id, ())),
                    |p, id| write!(p, "{:?}", id),
                    |p, (id, _)| pretty_print_patch(p, self, id),
                )
            })?;
            Ok(())
        })
    }
}

fn pretty_print_patch(
    p: &mut PrettyPrinter<impl Write>,
    patches: &Patches,
    id: PatchId,
) -> fmt::Result {
    p.add_struct("Patch", |p| {
        // add builtin fields
        p.add_field_with("base", |p| {
            write!(p, "{:?}", patches.get_patch_base_data(id).expect("patch must be valid"))
        })?;
        p.add_field_with("pcolor", |p| {
            write!(p, "{:?}", patches.get_patch_pcolor(id).expect("patch must be valid"))
        })?;

        // add custom fields
        for (field_name, field_desc) in patches.schema().custom_fields() {
            let AgentSchemaField::Other(ty) = patches.schema()[*field_desc] else {
                panic!("field at index {:?} should be a custom field", field_desc);
            };
            p.add_field_with(&field_name, |p| {
                fn print_field<T: Reflect + fmt::Debug>(
                    p: &mut PrettyPrinter<impl Write>,
                    patches: &Patches,
                    id: PatchId,
                    field: AgentFieldDescriptor,
                ) -> fmt::Result {
                    match patches.get_patch_field::<T>(id, field) {
                        None => write!(p, "None"),
                        Some(Either::Left(field)) => write!(p, "{:?}", field),
                        Some(Either::Right(field)) => write!(p, "fallback {:?}", field),
                    }
                }
                if ty == NlFloat::CONCRETE_TY {
                    print_field::<NlFloat>(p, patches, id, *field_desc)
                } else if ty == NlBool::CONCRETE_TY {
                    print_field::<NlBool>(p, patches, id, *field_desc)
                } else if ty == NlString::CONCRETE_TY {
                    print_field::<NlString>(p, patches, id, *field_desc)
                } else if ty == NlList::CONCRETE_TY {
                    print_field::<NlList>(p, patches, id, *field_desc)
                } else {
                    write!(p, "unknown type {:?}", ty)
                }
            })?;
        }
        Ok(())
    })
}

#[derive(Debug)]
pub struct PatchBaseData {
    pub position: Point,
    pub plabel: String, // FIXME consider using the netlogo version of string for this
    pub plabel_color: Color,
    // TODO add some way of tracking what turtles are on this patch.
}

static PATCH_BASE_DATA_TYPE_INFO: TypeInfo =
    TypeInfo::new::<PatchBaseData>(TypeInfoOptions { is_zeroable: false, mem_repr: None });

impl ConstTypeName for PatchBaseData {
    const TYPE_NAME: &'static str = "PatchBaseData";
}

unsafe impl Reflect for PatchBaseData {
    const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&PATCH_BASE_DATA_TYPE_INFO);
}

#[derive(Debug, Clone, Copy)]
pub enum PatchVarDesc {
    Pos,
    Pcolor,
    Custom(usize),
}

#[derive(Debug, Clone)]
pub struct PatchSchema {
    pcolor: AgentFieldDescriptor,
    field_groups: Vec<AgentSchemaFieldGroup>,
    custom_fields: Vec<(Arc<str>, AgentFieldDescriptor)>,
}

impl PatchSchema {
    pub fn new(
        pcolor_buffer_idx: u8,
        custom_fields: &[(&Arc<str>, ConcreteTy, u8)],
        avoid_occupancy_bitfield: &[u8],
    ) -> Self {
        // create field groups vector and add base data group
        let mut field_groups = Vec::new();
        field_groups.push(AgentSchemaFieldGroup {
            avoid_occupancy_bitfield: false,
            fields: vec![AgentSchemaField::BaseData],
        });

        // ensure field groups exist up to max needed index
        let max_buffer_idx =
            pcolor_buffer_idx.max(custom_fields.iter().map(|(_, _, idx)| *idx).max().unwrap_or(0));
        while field_groups.len() <= max_buffer_idx as usize {
            field_groups.push(AgentSchemaFieldGroup {
                avoid_occupancy_bitfield: false,
                fields: Vec::new(),
            });
        }

        // add pcolor field
        field_groups[pcolor_buffer_idx as usize]
            .fields
            .push(AgentSchemaField::Other(Color::CONCRETE_TY));

        // add custom fields and collect their descriptors
        let mut custom_field_descriptors = Vec::new();
        for (field_name, field_type, buffer_idx) in custom_fields {
            let field_idx = field_groups[*buffer_idx as usize].fields.len() as u8;
            field_groups[*buffer_idx as usize].fields.push(AgentSchemaField::Other(*field_type));
            custom_field_descriptors.push((
                Arc::clone(field_name),
                AgentFieldDescriptor { buffer_idx: *buffer_idx, field_idx },
            ));
        }

        // set avoid_occupancy_bitfield flags
        for &buffer_idx in avoid_occupancy_bitfield {
            assert!(
                (buffer_idx as usize) < field_groups.len(),
                "avoid_occupancy_bitfield index out of bounds"
            );
            field_groups[buffer_idx as usize].avoid_occupancy_bitfield = true;
        }

        // verify all field groups are non-empty
        for (i, group) in field_groups.iter().enumerate() {
            assert!(!group.fields.is_empty(), "field group at index {} is empty", i);
        }

        Self {
            pcolor: AgentFieldDescriptor { buffer_idx: pcolor_buffer_idx, field_idx: 0 },
            field_groups,
            custom_fields: custom_field_descriptors,
        }
    }

    pub fn make_row_schemas<const N: usize>(&self) -> [Option<RowSchema>; N] {
        super::agent_schema::make_row_schemas::<PatchBaseData, N>(&self.field_groups)
    }

    pub fn base_data(&self) -> AgentFieldDescriptor {
        AgentFieldDescriptor { buffer_idx: 0, field_idx: 0 }
    }

    pub fn pcolor(&self) -> AgentFieldDescriptor {
        self.pcolor
    }

    pub fn custom_fields(&self) -> &[(Arc<str>, AgentFieldDescriptor)] {
        &self.custom_fields
    }

    pub fn field_desc_and_offset(&self, var: PatchVarDesc) -> (AgentFieldDescriptor, usize) {
        match var {
            PatchVarDesc::Pos => (self.base_data(), offset_of!(PatchBaseData, position)),
            PatchVarDesc::Pcolor => (self.pcolor(), 0),
            PatchVarDesc::Custom(field_id) => (self.custom_fields()[field_id].1, 0),
        }
    }
}

impl Default for PatchSchema {
    fn default() -> Self {
        Self::new(0, &[], &[])
    }
}

impl Index<AgentFieldDescriptor> for PatchSchema {
    type Output = AgentSchemaField;

    fn index(&self, index: AgentFieldDescriptor) -> &Self::Output {
        &self.field_groups[index.buffer_idx as usize].fields[index.field_idx as usize]
    }
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
    let (buffer_idx, stride, field_offset) = {
        let (field_desc, additional_offset) = patch_schema.field_desc_and_offset(var);
        let (stride, field_offset) = stride_and_field_offset(patch_schema, field_desc);
        (field_desc.buffer_idx, stride, field_offset + additional_offset)
    };
    let buffer_offset =
        offset_of!(Patches, data) + (usize::from(buffer_idx) * size_of::<Option<RowBuffer>>());
    (buffer_offset, stride, field_offset)
}

pub fn patch_var_type(schema: &PatchSchema, var: PatchVarDesc) -> ConcreteTy {
    match var {
        PatchVarDesc::Pcolor => Color::CONCRETE_TY,
        PatchVarDesc::Pos => Point::CONCRETE_TY,
        PatchVarDesc::Custom(field) => {
            let AgentSchemaField::Other(ty) = schema[schema.custom_fields()[field].1] else {
                unreachable!("this is a custom field, so it cannot be part of the base data");
            };
            ty
        }
    }
}
