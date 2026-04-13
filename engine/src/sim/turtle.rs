//! Representation of turtles and breeds.
//!
//! From the perspective of this code, all turtles belong to a breed. Unbreeded
//! turtles belong to a special breed that acts like the `turtles` agentset.

use std::alloc::Layout;
use std::collections::BTreeMap;
use std::fmt::{self, Debug, Write};
use std::mem::offset_of;
use std::ops::Index;
use std::sync::Arc;

use derive_more::Display;
use derive_more::derive::{From, Into};
use either::Either;
use macro_reflect::{ReflectComponents, reflect};
use pretty_print::PrettyPrinter;

use crate::hir::{CustomVarDecl, TypeMapping};
use crate::mir::{self, HasDynPtr as _};
use crate::util::reflection::Reflect;
use crate::{
    sim::{
        agent_schema::{AgentFieldDescriptor, AgentSchemaField, AgentSchemaFieldGroup},
        color::Color,
        topology::{Heading, Point},
        value::{self, NlFloat, NlList, NlString, PackedAny, agentset::TurtleSet},
    },
    util::{
        gen_slot_tracker::{GenIndex, GenSlotTracker},
        reflection::Type,
        rng::Rng,
        row_buffer::{RowBuffer, RowSchema},
    },
};

pub const DEFAULT_BREED_NAME: &str = "TURTLES";

/// An ID for a turtle. When passing through FFI, this should be converted to a
/// `u64` first to prevent it from being represented as a pointer.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into, Default, ReflectComponents,
)]
// TODO specify to reflect that its contents are a u32
#[repr(transparent)]
pub struct TurtleId(pub GenIndex);

#[reflect(unsafe(is_zeroable), clone(copy))]
impl Reflect for TurtleId {}

impl TurtleId {
    pub const fn to_ffi(&self) -> u64 {
        self.0.to_ffi()
    }

    pub const fn from_ffi(value: u64) -> Self {
        Self(GenIndex::from_ffi(value))
    }

    pub const fn index(&self) -> usize {
        self.0.index as usize
    }
}

pub struct Turtles {
    /// The who number to be given to the next turtle; also how many turtles
    /// have been created since the last `clear-turtles`.
    next_who: NlFloat,
    /// Tracks which slots of the row buffers are occupied by turtles.
    slot_tracker: GenSlotTracker,
    /// The buffers that store the data for the turtle. Each turtle is
    /// represented by a row in all buffers. There are multiple buffers to
    /// allow for SoA-style data locality of certain fields.
    pub data: [Option<RowBuffer>; 4],
    /// Fallback storage for custom fields whose type doesn't match the
    /// compile-time type.
    fallback_custom_fields: BTreeMap<(TurtleId, AgentFieldDescriptor), PackedAny>,
    /// The fields of a turtle.
    turtle_schema: TurtleSchema,
    /// The number of turtles in the world.
    num_turtles: u64,
    /// The breeds of turtles.
    breeds: BTreeMap<TurtleBreedId, TurtleBreed>,
}

impl Turtles {
    pub fn new(turtle_schema: TurtleSchema, breeds: BTreeMap<TurtleBreedId, TurtleBreed>) -> Self {
        Self {
            next_who: NlFloat::new(0.0),
            slot_tracker: GenSlotTracker::new(),
            // TODO(wishlist) we should avoid having to remake the row schemas
            // if we can; we should reuse the ones from the compilation process
            // instead.
            data: turtle_schema.make_row_schemas().map(|s| s.map(RowBuffer::new)),
            fallback_custom_fields: BTreeMap::new(),
            turtle_schema,
            num_turtles: 0,
            breeds,
        }
    }

    pub fn schema(&self) -> &TurtleSchema {
        &self.turtle_schema
    }

    pub fn breeds(&self) -> &BTreeMap<TurtleBreedId, TurtleBreed> {
        &self.breeds
    }

    pub fn translate_who(&self, _who: NlFloat) -> TurtleId {
        todo!("TODO(mvp) use a lookup table to translate")
    }

    pub fn create_turtles(
        &mut self,
        breed: TurtleBreedId,
        count: u64,
        spawn_point: Point,
        next_int: &mut dyn Rng,
    ) -> TurtleSet {
        for buffer in self.data.iter_mut().filter_map(|b| b.as_mut()) {
            buffer.ensure_capacity((self.num_turtles + count) as usize);
        }

        let mut new_turtles: Vec<TurtleId> = Vec::new();
        for _ in 0..count {
            let idx = self.slot_tracker.allocate();
            let id = TurtleId(idx);
            let who = self.next_who;
            self.next_who += NlFloat::new(1.0);
            let color = Color::random(next_int);
            let heading = Heading::random(next_int);
            let shape_name = NlString::from_str("default"); // FIXME look up and use the breed's default shape

            // initialize base data
            let base_data = TurtleBaseData {
                who,
                breed,
                color,
                label: NlString::new(),
                label_color: color, // FIXME use a default label color
                hidden: false,
                size: value::NlFloat::new(1.0),
                shape_name,
            };
            self.data[0].as_mut().unwrap().row_mut(id.0.index as usize).set(0, base_data);

            // set builtin variables that aren't in the base data
            let heading_desc = self.turtle_schema.heading();
            self.data[heading_desc.buffer_idx as usize]
                .as_mut()
                .unwrap()
                .row_mut(id.0.index as usize)
                .set(heading_desc.field_idx as usize, heading);
            let position_desc = self.turtle_schema.position();
            self.data[position_desc.buffer_idx as usize]
                .as_mut()
                .unwrap()
                .row_mut(id.0.index as usize)
                .set(position_desc.field_idx as usize, spawn_point);

            // put in the default value for custom fields
            let custom_fields = self.breeds[&breed]
                .custom_variables
                .iter()
                .map(|&var_idx| self.turtle_schema.custom_fields()[var_idx].1);
            for field in custom_fields {
                let AgentSchemaField::Other(r#type) = self.turtle_schema[field] else {
                    panic!("field at index {:?} should be a custom field", field);
                };
                if r#type.is_zeroable {
                    self.data[field.buffer_idx as usize]
                        .as_mut()
                        .unwrap()
                        .row_mut(idx.index as usize)
                        .set_zero(field.field_idx as usize);
                } else {
                    self.fallback_custom_fields.insert((id, field), PackedAny::ZERO);
                }
            }

            new_turtles.push(id);
        }
        self.num_turtles += count;
        TurtleSet::new(new_turtles)
    }

    pub fn num_turtles(&self) -> u64 {
        self.num_turtles
    }

    pub fn turtle_ids(&self) -> impl Iterator<Item = TurtleId> + '_ {
        self.slot_tracker.iter().map(TurtleId)
    }

    /// Get a reference to a field of a turtle. Returns `None` if the
    /// turtle does not exist.
    pub fn get_turtle_field<T: Reflect + 'static>(
        &self,
        id: TurtleId,
        field: AgentFieldDescriptor,
    ) -> Option<Either<&T, &PackedAny>> {
        if !self.slot_tracker.has_key(id.0) {
            return None;
        }
        if let Some(field) = self.data[field.buffer_idx as usize]
            .as_ref()
            .expect("data buffer should exist")
            .row(id.0.index as usize)
            .get(field.field_idx as usize)
        {
            Some(Either::Left(field))
        } else {
            let fallback = self.fallback_custom_fields.get(&(id, field));
            Some(Either::Right(fallback?))
        }
    }

    pub fn get_turtle_base_data(&self, id: TurtleId) -> Option<&TurtleBaseData> {
        self.get_turtle_field(id, AgentFieldDescriptor::BASE_DATA)
            .map(|either| either.expect_left("base data should always exist in the row buffer"))
    }

    pub fn get_turtle_heading(&self, id: TurtleId) -> Option<&Heading> {
        self.get_turtle_field(id, self.turtle_schema.heading())
            .map(|either| either.expect_left("heading should always exist in the row buffer"))
    }

    pub fn get_turtle_position(&self, id: TurtleId) -> Option<&Point> {
        self.get_turtle_field(id, self.turtle_schema.position())
            .map(|either| either.expect_left("position should always exist in the row buffer"))
    }

    /// Get a mutable reference to a field of a turtle. Returns `None` if the
    /// turtle does not exist.
    pub fn get_turtle_field_mut<T: Reflect + 'static>(
        &mut self,
        id: TurtleId,
        field: AgentFieldDescriptor,
    ) -> Option<Either<&mut T, &mut PackedAny>> {
        if !self.slot_tracker.has_key(id.0) {
            return None;
        }
        if let Some(field) = self.data[field.buffer_idx as usize]
            .as_mut()
            .expect("data buffer should exist")
            .row_mut(id.0.index as usize)
            .get_mut(field.field_idx as usize)
        {
            Some(Either::Left(field))
        } else {
            let fallback = self.fallback_custom_fields.get_mut(&(id, field));
            Some(Either::Right(fallback?))
        }
    }

    pub fn get_turtle_base_data_mut(&mut self, id: TurtleId) -> Option<&mut TurtleBaseData> {
        self.get_turtle_field_mut(id, AgentFieldDescriptor::BASE_DATA)
            .map(|either| either.expect_left("base data should always exist in the row buffer"))
    }

    pub fn get_turtle_heading_mut(&mut self, id: TurtleId) -> Option<&mut Heading> {
        self.get_turtle_field_mut(id, self.turtle_schema.heading())
            .map(|either| either.expect_left("heading should always exist in the row buffer"))
    }

    pub fn get_turtle_position_mut(&mut self, id: TurtleId) -> Option<&mut Point> {
        self.get_turtle_field_mut(id, self.turtle_schema.position())
            .map(|either| either.expect_left("position should always exist in the row buffer"))
    }

    pub fn clear(&mut self) {
        self.slot_tracker.clear();
        self.next_who = NlFloat::new(0.0);
        self.fallback_custom_fields.clear();
        for buffer in self.data.iter_mut().filter_map(|b| b.as_mut()) {
            buffer.clear();
        }
    }

    pub fn mir_project_turtle_variable(
        builder: &mut mir::FunctionBuilder,
        type_mapping: &TypeMapping,
        turtles: mir::Place,
        turtle_id: mir::Place,
        var: TurtleVarDesc,
    ) -> mir::Place {
        const { assert!(size_of::<RowBuffer>() == size_of::<Option<RowBuffer>>()) };

        let (field_desc, offset) = type_mapping.turtle_schema().field_desc_and_offset(var);

        let offset_of_buffer = offset_of!(Self, data)
            + usize::from(field_desc.buffer_idx) * size_of::<Option<RowBuffer>>();
        let buffer_pl = turtles.proj(mir::Projection::Field { byte_offset: offset_of_buffer });
        let turtle_idx = builder.add_operation(
            Some("turtle_idx".into()),
            mir::Operation::UnaryOp {
                opcode: lir::UnaryOpcode::I64ToI32,
                operand: mir::PlaceOperand::Copy(turtle_id),
            },
        );
        let var_pl = RowBuffer::mir_project_field(
            builder,
            buffer_pl,
            turtle_idx,
            field_desc.field_idx as usize,
        );
        if let Some(offset) = offset { var_pl.proj_field(offset) } else { var_pl }
    }

    pub fn mir_type_from_schema(schema: &TurtleSchema) -> mir::MirType {
        // this code relies on the fact that the RowBuffer struct is niche
        // optimized so that an Option<RowBuffer> which is known to be Some can
        // be treated as a RowBuffer. A better solution would be to use the
        // offset_of! macro with the Some variant contents, but that is not yet
        // stable. This assertion serves as the closest approximation to
        // make sure that we can perform what is effectively transmutation
        const { assert!(size_of::<RowBuffer>() == size_of::<Option<RowBuffer>>()) };

        // we get 4 to match the number of buffers in the Turtles struct
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

        mir::MirTypeInfo::with_fields(Layout::new::<Self>(), fields)
    }
}

impl fmt::Debug for Turtles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut p = PrettyPrinter::new(f);
        let Turtles {
            next_who,
            slot_tracker: _,
            data: _,
            fallback_custom_fields: _,
            turtle_schema,
            num_turtles,
            breeds,
        } = self;
        p.add_struct("Turtles", |p| {
            p.add_field_with("breeds", |p| write!(p, "{:?}", breeds))?;
            p.add_field_with("turtle_schema", |p| write!(p, "{:?}", turtle_schema))?;
            p.add_field_with("num_turtles", |p| write!(p, "{}", num_turtles))?;
            p.add_field_with("next_who", |p| write!(p, "{:?}", next_who))?;
            p.add_field_with("turtles", |p| {
                p.add_map(
                    self.turtle_ids().map(|t| (t, ())),
                    |p, id| write!(p, "{:?}", id),
                    |p, (id, _)| pretty_print_turtle(p, self, id),
                )
            })?;
            Ok(())
        })
    }
}

fn pretty_print_turtle(
    p: &mut PrettyPrinter<impl Write>,
    turtles: &Turtles,
    id: TurtleId,
) -> fmt::Result {
    p.add_struct("Turtle", |p| {
        // add builtin fields
        p.add_field_with("base", |p| {
            write!(p, "{:?}", turtles.get_turtle_base_data(id).expect("turtle must be valid"))
        })?;
        p.add_field_with("pos", |p| {
            write!(p, "{:?}", turtles.get_turtle_position(id).expect("turtle must be valid"))
        })?;
        p.add_field_with("heading", |p| {
            write!(p, "{:?}", turtles.get_turtle_heading(id).expect("turtle must be valid"))
        })?;

        // add custom fields
        for (field_name, field_desc) in turtles.schema().custom_fields() {
            let AgentSchemaField::Other(ty) = &turtles.schema()[*field_desc] else {
                panic!("field at index {:?} should not be base data", field_desc);
            };
            p.add_field_with(field_name, |p| {
                fn print_field<T: Reflect + Debug + 'static>(
                    p: &mut PrettyPrinter<impl Write>,
                    turtles: &Turtles,
                    id: TurtleId,
                    field_desc: AgentFieldDescriptor,
                ) -> fmt::Result {
                    match turtles.get_turtle_field::<T>(id, field_desc) {
                        None => write!(p, "None"),
                        Some(Either::Left(field)) => write!(p, "{:?}", field),
                        Some(Either::Right(field)) => write!(p, "fallback {:?}", field),
                    }
                }
                if ty.is::<NlFloat>() {
                    print_field::<NlFloat>(p, turtles, id, *field_desc)
                } else if ty.is::<bool>() {
                    print_field::<bool>(p, turtles, id, *field_desc)
                } else if ty.is::<NlString>() {
                    print_field::<NlString>(p, turtles, id, *field_desc)
                } else if ty.is::<NlList>() {
                    print_field::<NlList>(p, turtles, id, *field_desc)
                } else {
                    write!(p, "unknown type {:?}", ty)
                }
            })?;
        }
        Ok(())
    })
}

#[derive(Debug, Clone, ReflectComponents)]
#[repr(C)]
pub struct TurtleBaseData {
    #[mir_accessible]
    pub who: NlFloat,
    #[mir_accessible]
    pub breed: TurtleBreedId,
    /// The shape of this turtle due to its breed. This may or may not be the
    /// default shape of the turtle's breed.
    #[mir_accessible]
    pub shape_name: NlString,
    #[mir_accessible]
    pub color: Color,
    #[mir_accessible]
    pub label: NlString,
    #[mir_accessible]
    pub label_color: Color,
    #[mir_accessible]
    pub hidden: bool,
    #[mir_accessible]
    pub size: value::NlFloat,
}

#[reflect]
impl Reflect for TurtleBaseData {}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Display, PartialOrd, Ord, ReflectComponents)]
#[display("{_0}")]
pub struct TurtleBreedId(pub u32);

#[reflect]
impl Reflect for TurtleBreedId {}

#[derive(Debug)]
pub struct TurtleBreed {
    pub name: Arc<str>,
    pub singular_name: Arc<str>,
    /// The indices of the custom variables that are active for this breed.
    pub custom_variables: Vec<usize>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TurtleVarDesc {
    Who,
    Size,
    Color,
    /// The position of the turtle, containing both the x and y coordinates.
    /// While not visible to the user-level code, this is how it is actually
    /// stored, and the engine may emit code that reads/writes the entire field
    /// at once.
    Pos,
    /// The x coordinate of the position of the turtle. Aliases the `Pos`
    /// variable.
    Xcor,
    /// The y coordinate of the position of the turtle. Aliases the `Pos`
    /// variable.
    Ycor,
    /// The nth custom field of the turtle.
    Custom(usize),
    // TODO(mvp) add other builtin variables
}

impl TurtleVarDesc {
    pub fn pretty_print(
        &self,
        p: &mut PrettyPrinter<impl Write>,
        custom_turtle_vars: &[CustomVarDecl],
    ) -> fmt::Result {
        match self {
            TurtleVarDesc::Who => write!(p, "WHO"),
            TurtleVarDesc::Size => write!(p, "SIZE"),
            TurtleVarDesc::Color => write!(p, "COLOR"),
            TurtleVarDesc::Pos => write!(p, "POS"),
            TurtleVarDesc::Xcor => write!(p, "XCOR"),
            TurtleVarDesc::Ycor => write!(p, "YCOR"),
            TurtleVarDesc::Custom(field) => {
                write!(p, "{}#{}", field, custom_turtle_vars[*field].name)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TurtleSchema {
    position: AgentFieldDescriptor,
    heading: AgentFieldDescriptor,
    field_groups: Vec<AgentSchemaFieldGroup>,
    custom_fields: Vec<(Arc<str>, AgentFieldDescriptor)>,
}

pub enum TurtleFieldGroupElement {
    BaseData,
    Position,
    Heading,
    Custom { name: Arc<str>, ty: Type },
}

pub struct TurtleFieldGroup {
    pub avoid_occupancy_bitfield: bool,
    pub fields: Vec<TurtleFieldGroupElement>,
}

impl TurtleSchema {
    pub fn new_with_field_groups(field_groups_spec: Vec<TurtleFieldGroup>) -> Self {
        let mut base_data = None;
        let mut position = None;
        let mut heading = None;
        let mut field_groups = Vec::new();
        let mut custom_fields = Vec::new();

        for (buffer_idx, TurtleFieldGroup { avoid_occupancy_bitfield, fields }) in
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
                    TurtleFieldGroupElement::BaseData => {
                        if base_data.is_some() {
                            panic!("base data cannot be included more than once");
                        }
                        base_data = Some(current_field_desc);
                        agent_schema_field_group.fields.push(AgentSchemaField::BaseData);
                    }
                    TurtleFieldGroupElement::Position => {
                        if position.is_some() {
                            panic!("position cannot be included more than once");
                        }
                        position = Some(current_field_desc);
                        agent_schema_field_group.fields.push(AgentSchemaField::Other(Point::TYPE));
                    }
                    TurtleFieldGroupElement::Heading => {
                        if heading.is_some() {
                            panic!("heading cannot be included more than once");
                        }
                        heading = Some(current_field_desc);
                        agent_schema_field_group
                            .fields
                            .push(AgentSchemaField::Other(Heading::TYPE));
                    }
                    TurtleFieldGroupElement::Custom { name, ty } => {
                        custom_fields.push((name, current_field_desc));
                        agent_schema_field_group.fields.push(AgentSchemaField::Other(ty));
                    }
                }
            }
            field_groups.push(agent_schema_field_group);
        }

        Self {
            position: position.expect("position must be present"),
            heading: heading.expect("heading must be present"),
            field_groups,
            custom_fields,
        }
    }

    pub fn make_row_schemas<const N: usize>(&self) -> [Option<RowSchema>; N] {
        super::agent_schema::make_row_schemas::<TurtleBaseData, N>(&self.field_groups)
    }

    pub fn base_data(&self) -> AgentFieldDescriptor {
        AgentFieldDescriptor { buffer_idx: 0, field_idx: 0 }
    }

    pub fn heading(&self) -> AgentFieldDescriptor {
        self.heading
    }

    pub fn position(&self) -> AgentFieldDescriptor {
        self.position
    }

    pub fn custom_fields(&self) -> &[(Arc<str>, AgentFieldDescriptor)] {
        &self.custom_fields
    }

    pub fn field_desc_and_offset(
        &self,
        var: TurtleVarDesc,
    ) -> (AgentFieldDescriptor, Option<usize>) {
        match var {
            TurtleVarDesc::Custom(field_id) => (self.custom_fields()[field_id].1, None),
            TurtleVarDesc::Who => (self.base_data(), Some(offset_of!(TurtleBaseData, who))),
            TurtleVarDesc::Size => (self.base_data(), Some(offset_of!(TurtleBaseData, size))),
            TurtleVarDesc::Color => (self.base_data(), Some(offset_of!(TurtleBaseData, color))),
            TurtleVarDesc::Pos => (self.position(), None),
            TurtleVarDesc::Xcor => (self.position(), Some(offset_of!(Point, x))),
            TurtleVarDesc::Ycor => (self.position(), Some(offset_of!(Point, y))),
        }
    }

    pub fn var_type(&self, var: TurtleVarDesc) -> Type {
        match var {
            TurtleVarDesc::Who => NlFloat::TYPE,
            TurtleVarDesc::Color => Color::TYPE,
            TurtleVarDesc::Size => NlFloat::TYPE,
            TurtleVarDesc::Pos => Point::TYPE,
            TurtleVarDesc::Xcor => NlFloat::TYPE,
            TurtleVarDesc::Ycor => NlFloat::TYPE,
            TurtleVarDesc::Custom(field) => {
                let var_desc = self.custom_fields()[field].1;
                let AgentSchemaField::Other(ty) = self[var_desc] else {
                    unreachable!("this is a custom field, so it cannot be part of the base data");
                };
                ty
            }
        }
    }
}

impl Default for TurtleSchema {
    fn default() -> Self {
        Self::new_with_field_groups(vec![TurtleFieldGroup {
            avoid_occupancy_bitfield: false,
            fields: vec![
                TurtleFieldGroupElement::BaseData,
                TurtleFieldGroupElement::Position,
                TurtleFieldGroupElement::Heading,
            ],
        }])
    }
}

impl Index<AgentFieldDescriptor> for TurtleSchema {
    type Output = AgentSchemaField;

    fn index(&self, index: AgentFieldDescriptor) -> &Self::Output {
        &self.field_groups[index.buffer_idx as usize].fields[index.field_idx as usize]
    }
}

pub fn turtle_var_type(schema: &TurtleSchema, var: TurtleVarDesc) -> Type {
    match var {
        TurtleVarDesc::Who => NlFloat::TYPE,
        TurtleVarDesc::Color => Color::TYPE,
        TurtleVarDesc::Size => NlFloat::TYPE,
        TurtleVarDesc::Pos => Point::TYPE,
        TurtleVarDesc::Xcor => NlFloat::TYPE,
        TurtleVarDesc::Ycor => NlFloat::TYPE,
        TurtleVarDesc::Custom(field) => {
            let AgentSchemaField::Other(ty) = schema[schema.custom_fields()[field].1] else {
                unreachable!("this is a custom field, so it cannot be part of the base data");
            };
            ty
        }
    }
}

// TODO(test) write tests for turtle initialization and access
