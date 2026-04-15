use std::{
    alloc::Layout,
    collections::HashMap,
    fmt::{self, Write},
    mem::offset_of,
    ops::Index,
    sync::Arc,
};

use derive_more::derive::From;
use either::Either;
use macro_reflect::{MirReflect, reflect};
use pretty_print::PrettyPrinter;

use super::topology::Point;
use crate::{
    hir::{CustomVarDecl, HirToMirFnBuilder, TypeMapping},
    mir::{self, HasDynPtr as _},
    sim::{
        agent_schema::{AgentFieldDescriptor, AgentSchemaField, AgentSchemaFieldGroup},
        color,
        topology::{PointInt, TopologySpec},
        value::{BoxedAny, NlFloat, NlList, NlString, PackedAny},
    },
    util::{
        reflection::{Reflect, Type},
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, MirReflect)]
// TODO reflection contents
#[repr(transparent)]
pub struct PatchId(pub u32);

#[reflect(unsafe(is_zeroable), clone(copy))]
impl Reflect for PatchId {}

impl Default for PatchId {
    fn default() -> Self {
        Self(u32::MAX)
    }
}

/// Exactly the same as [`PatchId`], but it can represent "nobody" at the -1
/// value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, MirReflect)]
// TODO reflection contents
#[repr(transparent)]
pub struct OptionPatchId(pub u32);

#[reflect(clone(copy))]
impl Reflect for OptionPatchId {}

impl OptionPatchId {
    pub const NOBODY: Self = Self(u32::MAX);

    /// Writes the MIR statements to check if the given operand is nobody.
    /// The result is stored in the given out local.
    pub fn write_check_nobody(
        builder: &mut HirToMirFnBuilder,
        negate: bool,
        operand: mir::Place,
    ) -> mir::LocalId {
        let sentinel_pl = builder.mir.add_operation(
            None,
            mir::Operation::Const { value: BoxedAny::new(OptionPatchId::NOBODY) },
        );
        let opcode = if negate { lir::BinaryOpcode::INeq } else { lir::BinaryOpcode::IEq };
        let result = builder.mir.add_operation(
            None,
            mir::Operation::BinaryOp {
                opcode,
                lhs: mir::PlaceOperand::Copy(operand),
                rhs: mir::PlaceOperand::Copy(sentinel_pl.place()),
            },
        );
        result
    }
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
        patches.clear_patch_variables(topology_spec);

        patches
    }

    pub fn schema(&self) -> &PatchSchema {
        &self.patch_schema
    }

    /// Get a reference to a field of a patch. Returns `None` if the
    /// patch does not exist.
    pub fn get_patch_field<T: Reflect + 'static>(
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

    pub fn get_patch_pcolor(&self, id: PatchId) -> Option<&NlFloat> {
        self.get_patch_field(id, self.patch_schema.pcolor())
            .map(|either| either.expect_left("pcolor should always exist in the row buffer"))
    }

    /// Get a mutable reference to a field of a patch. Returns `None` if the
    /// patch does not exist.
    pub fn get_patch_field_mut<T: Reflect + 'static>(
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

    pub fn get_patch_pcolor_mut(&mut self, id: PatchId) -> Option<&mut NlFloat> {
        self.get_patch_field_mut(id, self.patch_schema.pcolor())
            .map(|either| either.expect_left("pcolor should always exist in the row buffer"))
    }

    pub fn set_patch_field<T: Reflect + 'static>(
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
            .set(field.field_idx as usize, value);
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
    pub fn patch_field_as_mut_array<T: Copy + Reflect + 'static>(
        &mut self,
        field: AgentFieldDescriptor,
    ) -> &mut [T] {
        self.data[field.buffer_idx as usize].as_mut().unwrap().as_mut_array()
    }

    /// Resets all patch variables to their default values.
    pub fn clear_patch_variables(&mut self, topology_spec: &TopologySpec) {
        self.fallback_custom_fields.clear();

        let TopologySpec { min_pxcor, max_pycor, patches_height, patches_width, .. } =
            topology_spec;
        for j in 0..*patches_height {
            for i in 0..*patches_width {
                let x = min_pxcor + i;
                let y = max_pycor - j;
                let position = Point { x: x.into(), y: y.into() };
                // topology_spec.patch_at(position) should just return an
                // increasing index anyway but this is more robust (even though
                // it's literally the same thing just requiring more
                // optimization)
                let id = topology_spec.patch_at(PointInt { x, y });

                // initialize base data
                let base_data = PatchBaseData {
                    position,
                    plabel: NlString::new(),
                    plabel_color: color::BLACK, // FIXME use a more sensible default
                };
                self.data[0].as_mut().unwrap().row_mut(id.0 as usize).set(0, base_data);

                // initialize other builtins
                let pcolor_desc = self.patch_schema.pcolor();

                self.data[pcolor_desc.buffer_idx as usize]
                    .as_mut()
                    .unwrap()
                    .row_mut(id.0 as usize)
                    .set_zero(pcolor_desc.field_idx as usize);

                // initialize custom fields
                for &(_, field) in self.patch_schema.custom_fields() {
                    let AgentSchemaField::Other(r#type) = self.patch_schema[field] else {
                        panic!("field at index {:?} should be a custom field", field);
                    };
                    if r#type.is_zeroable {
                        self.data[field.buffer_idx as usize]
                            .as_mut()
                            .unwrap()
                            .row_mut(id.0 as usize)
                            .set_zero(field.field_idx as usize);
                    } else {
                        self.fallback_custom_fields.insert((id, field), PackedAny::ZERO);
                    }
                }
                // TODO(wishlist) can reduce code duplication by using a helper
                // function for initialization of the custom fields of turtles,
                // patches, and links
            }
        }
    }

    pub fn patch_ids(&self) -> impl Iterator<Item = PatchId> + '_ {
        (0..self.num_patches).map(PatchId)
    }

    pub fn mir_project_patch_variable(
        builder: &mut mir::FunctionBuilder,
        type_mapping: &TypeMapping,
        patches: mir::Place,
        patch_id: mir::Place,
        var: PatchVarDesc,
    ) -> mir::Place {
        const { assert!(size_of::<RowBuffer>() == size_of::<Option<RowBuffer>>()) };

        let (field_desc, offset) = type_mapping.patch_schema().field_desc_and_offset(var);

        let offset_of_buffer = offset_of!(Self, data)
            + usize::from(field_desc.buffer_idx) * size_of::<Option<RowBuffer>>();
        let buffer_pl = patches.proj(mir::Projection::Field { byte_offset: offset_of_buffer });
        let var_pl = RowBuffer::mir_project_field(
            builder,
            buffer_pl,
            patch_id.unwrap_local(),
            field_desc.field_idx as usize,
        );

        if let Some(offset) = offset { var_pl.proj_field(offset) } else { var_pl }
    }

    pub fn mir_type_from_schema(schema: &PatchSchema) -> mir::MirType {
        // this code relies on the fact that the RowBuffer struct is niche
        // optimized so that an Option<RowBuffer> which is known to be Some can
        // be treated as a RowBuffer. A better solution would be to use the
        // offset_of! macro with the Some variant contents, but that is not yet
        // stable. This assertion serves as the closest approximation to
        // make sure that we can perform what is effectively transmutation
        const { assert!(size_of::<RowBuffer>() == size_of::<Option<RowBuffer>>()) };

        // we get 4 to match the number of buffers in the Patches struct
        let buffer_types: [Option<mir::MirType>; 4] = schema
            .make_row_schemas()
            .map(|schema| schema.map(|s| RowBuffer::self_mir_type_from_metadata(&s)));
        let fields = buffer_types
            .into_iter()
            .enumerate()
            .filter_map(|(buffer_idx, ty)| {
                let ty = ty?;
                let offset = offset_of!(Self, data) + buffer_idx * size_of::<Option<RowBuffer>>();
                Some((offset, ty))
            })
            .collect();

        mir::MirType::new_struct(Layout::new::<Self>(), fields)
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
            p.add_field_with(field_name, |p| {
                fn print_field<T: Reflect + fmt::Debug + 'static>(
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
                if ty.is::<NlFloat>() {
                    print_field::<NlFloat>(p, patches, id, *field_desc)
                } else if ty.is::<bool>() {
                    print_field::<bool>(p, patches, id, *field_desc)
                } else if ty.is::<NlString>() {
                    print_field::<NlString>(p, patches, id, *field_desc)
                } else if ty.is::<NlList>() {
                    print_field::<NlList>(p, patches, id, *field_desc)
                } else {
                    write!(p, "unknown type {:?}", ty)
                }
            })?;
        }
        Ok(())
    })
}

#[derive(Debug, Clone, MirReflect)]
pub struct PatchBaseData {
    #[mir_accessible]
    pub position: Point,
    #[mir_accessible]
    pub plabel: NlString,
    #[mir_accessible]
    pub plabel_color: NlFloat,
    // TODO add some way of tracking what turtles are on this patch.
}

#[reflect(special_mir_type)]
impl Reflect for PatchBaseData {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PatchVarDesc {
    Pos,
    Pcolor,
    Custom(usize),
}

impl PatchVarDesc {
    pub fn pretty_print(
        &self,
        p: &mut PrettyPrinter<impl Write>,
        custom_patch_vars: &[CustomVarDecl],
    ) -> fmt::Result {
        match self {
            PatchVarDesc::Pos => write!(p, "POS"),
            PatchVarDesc::Pcolor => write!(p, "PCOLOR"),
            PatchVarDesc::Custom(field) => {
                write!(p, "{}#{}", field, custom_patch_vars[*field].name)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PatchSchema {
    base_data: AgentFieldDescriptor,
    pcolor: AgentFieldDescriptor,
    field_groups: Vec<AgentSchemaFieldGroup>,
    custom_fields: Vec<(Arc<str>, AgentFieldDescriptor)>,
}

pub enum PatchFieldGroupElement {
    BaseData,
    Pcolor,
    Custom { name: Arc<str>, ty: Type },
}

pub struct PatchFieldGroup {
    pub avoid_occupancy_bitfield: bool,
    pub fields: Vec<PatchFieldGroupElement>,
}

impl PatchSchema {
    pub fn new_with_field_groups(field_groups_spec: Vec<PatchFieldGroup>) -> Self {
        let mut base_data = None;
        let mut pcolor = None;
        let mut field_groups = Vec::new();
        let mut custom_fields = Vec::new();

        for (buffer_idx, PatchFieldGroup { avoid_occupancy_bitfield, fields }) in
            field_groups_spec.into_iter().enumerate()
        {
            let mut agent_schema_field_group =
                AgentSchemaFieldGroup { avoid_occupancy_bitfield, fields: Vec::new() };
            for (field_idx, field) in fields.into_iter().enumerate() {
                let current_field_desc = AgentFieldDescriptor {
                    buffer_idx: buffer_idx.try_into().unwrap(),
                    field_idx: field_idx.try_into().unwrap(),
                };
                match field {
                    PatchFieldGroupElement::BaseData => {
                        if base_data.is_some() {
                            panic!("base data cannot be included more than once");
                        }
                        base_data = Some(current_field_desc);
                        agent_schema_field_group.fields.push(AgentSchemaField::BaseData);
                    }
                    PatchFieldGroupElement::Pcolor => {
                        if pcolor.is_some() {
                            panic!("pcolor cannot be included more than once");
                        }
                        pcolor = Some(current_field_desc);
                        agent_schema_field_group
                            .fields
                            .push(AgentSchemaField::Other(NlFloat::TYPE));
                    }
                    PatchFieldGroupElement::Custom { name, ty } => {
                        custom_fields.push((name, current_field_desc));
                        agent_schema_field_group.fields.push(AgentSchemaField::Other(ty));
                    }
                };
            }
            field_groups.push(agent_schema_field_group);
        }

        Self {
            base_data: base_data.expect("base data must be present"),
            pcolor: pcolor.expect("pcolor must be present"),
            field_groups,
            custom_fields,
        }
    }

    pub fn make_row_schemas<const N: usize>(&self) -> [Option<RowSchema>; N] {
        super::agent_schema::make_row_schemas::<PatchBaseData, N>(&self.field_groups)
    }

    pub fn base_data(&self) -> AgentFieldDescriptor {
        self.base_data
    }

    pub fn pcolor(&self) -> AgentFieldDescriptor {
        self.pcolor
    }

    pub fn custom_fields(&self) -> &[(Arc<str>, AgentFieldDescriptor)] {
        &self.custom_fields
    }

    pub fn field_desc_and_offset(
        &self,
        var: PatchVarDesc,
    ) -> (AgentFieldDescriptor, Option<usize>) {
        match var {
            PatchVarDesc::Pos => (self.base_data(), Some(offset_of!(PatchBaseData, position))),
            PatchVarDesc::Pcolor => (self.pcolor(), None),
            PatchVarDesc::Custom(field_id) => (self.custom_fields()[field_id].1, None),
        }
    }
}

impl Default for PatchSchema {
    fn default() -> Self {
        Self::new_with_field_groups(vec![PatchFieldGroup {
            avoid_occupancy_bitfield: false,
            fields: vec![PatchFieldGroupElement::BaseData, PatchFieldGroupElement::Pcolor],
        }])
    }
}

impl Index<AgentFieldDescriptor> for PatchSchema {
    type Output = AgentSchemaField;

    fn index(&self, index: AgentFieldDescriptor) -> &Self::Output {
        &self.field_groups[index.buffer_idx as usize].fields[index.field_idx as usize]
    }
}

pub fn patch_var_type(schema: &PatchSchema, var: PatchVarDesc) -> Type {
    match var {
        PatchVarDesc::Pcolor => NlFloat::TYPE,
        PatchVarDesc::Pos => Point::TYPE,
        PatchVarDesc::Custom(field) => {
            let AgentSchemaField::Other(ty) = schema[schema.custom_fields()[field].1] else {
                unreachable!("this is a custom field, so it cannot be part of the base data");
            };
            ty
        }
    }
}
