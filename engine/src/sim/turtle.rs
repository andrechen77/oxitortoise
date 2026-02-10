//! Representation of turtles and breeds.
//!
//! From the perspective of this code, all turtles belong to a breed. Unbreeded
//! turtles belong to a special breed that acts like the `turtles` agentset.

use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::mem::offset_of;
use std::ops::Index;
use std::rc::Rc;

use derive_more::derive::{From, Into};
use either::Either;
use slotmap::SecondaryMap;

use crate::mir;
use crate::sim::agent_schema::{AgentFieldDescriptor, AgentSchemaField, AgentSchemaFieldGroup};
use crate::sim::topology::Heading;
use crate::sim::value::agentset::TurtleSet;
use crate::sim::value::{NlFloat, PackedAny};
use crate::util::gen_slot_tracker::{GenIndex, GenSlotTracker};
use crate::util::reflection::{ConcreteTy, ConstTypeName, Reflect, TypeInfo, TypeInfoOptions};
use crate::util::row_buffer::{RowBuffer, RowSchema};
use crate::{
    sim::{color::Color, topology::Point, value},
    util::rng::Rng,
};

pub const DEFAULT_BREED_NAME: &str = "TURTLES";

/// The who number of a turtle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
#[repr(transparent)]
pub struct TurtleWho(pub u64);

impl TurtleWho {
    fn take_next(&mut self) -> Self {
        let who = *self;
        self.0 += 1;
        who
    }
}

impl fmt::Display for TurtleWho {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(turtle {})", self.0)
    }
}

/// An ID for a turtle. When passing through FFI, this should be converted to a
/// `u64` first to prevent it from being represented as a pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Into, Default)]
#[repr(transparent)]
pub struct TurtleId(pub GenIndex);

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

static TURTLE_ID_TYPE_INFO: TypeInfo = TypeInfo::new::<TurtleId>(TypeInfoOptions {
    is_zeroable: false,
    mem_repr: Some(&[(0, lir::MemOpType::I64)]),
});

unsafe impl Reflect for TurtleId {
    const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&TURTLE_ID_TYPE_INFO);
}

impl ConstTypeName for TurtleId {
    const TYPE_NAME: &'static str = "TurtleId";
}

#[derive(Debug)]
pub struct Turtles {
    /// The who number to be given to the next turtle; also how many turtles
    /// have been created since the last `clear-turtles`.
    next_who: TurtleWho,
    /// Tracks which slots of the row buffers are occupied by turtles.
    slot_tracker: GenSlotTracker,
    /// The buffers that store the data for the turtle. Each turtle is
    /// represented by a row in all buffers. There are multiple buffers to
    /// allow for SoA-style data locality of certain fields.
    pub data: [Option<RowBuffer>; 4],
    /// Fallback storage for custom fields whose type doesn't match the
    /// compile-time type.
    fallback_custom_fields: HashMap<(TurtleId, AgentFieldDescriptor), PackedAny>,
    /// The fields of a turtle.
    turtle_schema: TurtleSchema,
    /// The number of turtles in the world.
    num_turtles: u64,
    // TODO(mvp) this should be a secondary map, using the breed ids generated
    // during MIR creation
    /// The breeds of turtles.
    breeds: SecondaryMap<BreedId, Breed>,
}

impl Turtles {
    pub fn new(turtle_schema: TurtleSchema, breeds: SecondaryMap<BreedId, Breed>) -> Self {
        Self {
            next_who: TurtleWho::default(),
            slot_tracker: GenSlotTracker::new(),
            // TODO(wishlist) we should avoid having to remake the row schemas
            // if we can; we should reuse the ones from the compilation process
            // instead.
            data: turtle_schema.make_row_schemas().map(|s| s.map(RowBuffer::new)),
            fallback_custom_fields: HashMap::new(),
            turtle_schema,
            num_turtles: 0,
            breeds,
        }
    }

    pub fn schema(&self) -> &TurtleSchema {
        &self.turtle_schema
    }

    pub fn get_breed(&self, id: BreedId) -> &Breed {
        &self.breeds[id]
    }

    pub fn breeds(&self) -> &SecondaryMap<BreedId, Breed> {
        &self.breeds
    }

    pub fn translate_who(&self, _who: TurtleWho) -> TurtleId {
        todo!("TODO(mvp) use a lookup table to translate")
    }

    pub fn create_turtles(
        &mut self,
        breed: BreedId,
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
            let who = self.next_who.take_next();
            let color = Color::random(next_int);
            let heading = Heading::random(next_int);
            let shape_name = "default".to_owned(); // FIXME look up and use the breed's default shape

            // initialize base data
            let base_data = TurtleBaseData {
                who,
                breed,
                color,
                label: String::new(),
                label_color: color, // FIXME use a default label color
                hidden: false,
                size: value::NlFloat::new(1.0),
                shape_name,
            };
            self.data[0].as_mut().unwrap().row_mut(id.0.index as usize).insert(0, base_data);

            // set builtin variables that aren't in the base data
            let heading_desc = self.turtle_schema.heading();
            self.data[heading_desc.buffer_idx as usize]
                .as_mut()
                .unwrap()
                .row_mut(id.0.index as usize)
                .insert(heading_desc.field_idx as usize, heading);
            let position_desc = self.turtle_schema.position();
            self.data[position_desc.buffer_idx as usize]
                .as_mut()
                .unwrap()
                .row_mut(id.0.index as usize)
                .insert(position_desc.field_idx as usize, spawn_point);

            // put in the default value for custom fields
            let custom_fields = &self.breeds[breed].active_custom_fields;
            for &field in custom_fields {
                let AgentSchemaField::Other(r#type) = &self.turtle_schema[field] else {
                    panic!("field at index {:?} should be a custom field", field);
                };
                if r#type.info().is_zeroable {
                    self.data[field.buffer_idx as usize]
                        .as_mut()
                        .unwrap()
                        .row_mut(idx.index as usize)
                        .insert_zeroable(field.field_idx as usize);
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
    pub fn get_turtle_field<T: Reflect>(
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
    pub fn get_turtle_field_mut<T: Reflect>(
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
        self.next_who = TurtleWho::default();
        self.fallback_custom_fields.clear();
        for buffer in self.data.iter_mut().filter_map(|b| b.as_mut()) {
            buffer.clear();
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct TurtleBaseData {
    pub who: TurtleWho,
    pub breed: BreedId,
    /// The shape of this turtle due to its breed. This may or may not be the
    /// default shape of the turtle's breed.
    pub shape_name: String, // FIXME consider using the netlogo version of string for this
    pub color: Color,
    pub label: String, // FIXME consider using the netlogo version of string for this
    pub label_color: Color,
    pub hidden: bool,
    pub size: value::NlFloat,
}

static TURTLE_BASE_DATA_TYPE_INFO: TypeInfo =
    TypeInfo::new::<TurtleBaseData>(TypeInfoOptions { is_zeroable: false, mem_repr: None });

unsafe impl Reflect for TurtleBaseData {
    const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&TURTLE_BASE_DATA_TYPE_INFO);
}

impl ConstTypeName for TurtleBaseData {
    const TYPE_NAME: &'static str = "TurtleBaseData";
}

slotmap::new_key_type! {
    /// An ID for a breed.
    pub struct BreedId;
}

#[derive(Debug)]
pub struct Breed {
    pub name: Rc<str>,
    pub singular_name: Rc<str>,
    /// Which fields of the turtle record are active for this breed.
    pub active_custom_fields: Vec<AgentFieldDescriptor>,
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

#[derive(Debug, Clone)]
pub struct TurtleSchema {
    position: AgentFieldDescriptor,
    heading: AgentFieldDescriptor,
    field_groups: Vec<AgentSchemaFieldGroup>,
    custom_fields: Vec<AgentFieldDescriptor>,
}

impl TurtleSchema {
    pub fn new(
        heading_buffer_idx: u8,
        position_buffer_idx: u8,
        custom_fields: &[(ConcreteTy, u8)],
        avoid_occupancy_bitfield: &[u8],
    ) -> Self {
        // create field groups vector and add base data group
        let mut field_groups = Vec::new();
        field_groups.push(AgentSchemaFieldGroup {
            avoid_occupancy_bitfield: false,
            fields: vec![AgentSchemaField::BaseData],
        });

        // ensure field groups exist up to max needed index
        let max_buffer_idx = heading_buffer_idx
            .max(position_buffer_idx)
            .max(custom_fields.iter().map(|(_, idx)| *idx).max().unwrap_or(0));
        while field_groups.len() <= max_buffer_idx as usize {
            field_groups.push(AgentSchemaFieldGroup {
                avoid_occupancy_bitfield: false,
                fields: Vec::new(),
            });
        }

        // add heading and position fields
        let heading_group = &mut field_groups[heading_buffer_idx as usize];
        let heading_field_idx = heading_group.fields.len() as u8;
        heading_group.fields.push(AgentSchemaField::Other(Heading::CONCRETE_TY));
        let position_group = &mut field_groups[position_buffer_idx as usize];
        let position_field_idx = position_group.fields.len() as u8;
        position_group.fields.push(AgentSchemaField::Other(Point::CONCRETE_TY));

        // add custom fields
        let mut custom_field_descriptors = Vec::new();
        for (field_type, buffer_idx) in custom_fields {
            let field_group = &mut field_groups[usize::from(*buffer_idx)];
            let idx_within_buffer = field_group.fields.len();
            field_group.fields.push(AgentSchemaField::Other(*field_type));
            custom_field_descriptors.push(AgentFieldDescriptor {
                buffer_idx: *buffer_idx,
                field_idx: idx_within_buffer as u8,
            });
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
            heading: AgentFieldDescriptor {
                buffer_idx: heading_buffer_idx,
                field_idx: heading_field_idx,
            },
            position: AgentFieldDescriptor {
                buffer_idx: position_buffer_idx,
                field_idx: position_field_idx,
            },
            field_groups,
            custom_fields: custom_field_descriptors,
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

    pub fn custom_fields(&self) -> &[AgentFieldDescriptor] {
        &self.custom_fields
    }

    pub fn field_desc_and_offset(&self, var: TurtleVarDesc) -> (AgentFieldDescriptor, usize) {
        match var {
            TurtleVarDesc::Custom(field_id) => (self.custom_fields()[field_id], 0),
            TurtleVarDesc::Who => (self.base_data(), offset_of!(TurtleBaseData, who)),
            TurtleVarDesc::Size => (self.base_data(), offset_of!(TurtleBaseData, size)),
            TurtleVarDesc::Color => (self.base_data(), offset_of!(TurtleBaseData, color)),
            TurtleVarDesc::Pos => (self.position(), 0),
            TurtleVarDesc::Xcor => (self.position(), offset_of!(Point, x)),
            TurtleVarDesc::Ycor => (self.position(), offset_of!(Point, y)),
        }
    }
}

impl Default for TurtleSchema {
    fn default() -> Self {
        Self::new(0, 0, &[], &[])
    }
}

impl Index<AgentFieldDescriptor> for TurtleSchema {
    type Output = AgentSchemaField;

    fn index(&self, index: AgentFieldDescriptor) -> &Self::Output {
        &self.field_groups[index.buffer_idx as usize].fields[index.field_idx as usize]
    }
}

/// Returns a tuple indicating how to access a given variable given a pointer
/// to [`Turtles`]. The first element is the byte offset from the start of the
/// [`Turtles`] struct to the pointer to row buffer containing the variable.
/// The second element is the stride of the each row in that buffer; each agent
/// gets one row. The third element is the byte offset from the start of the
/// row to the required field.
///
/// ```ignore
/// let mir: &mir::Program;
/// let turtles: &Turtles;
/// let var_desc: TurtleVarDesc;
/// let (buffer_offset, stride, field_offset) = calc_turtle_var_offset(mir, var_desc);
/// let ptr_turtles = turtles as *const u8;
/// let field = *(*ptr_turtles.byte_add(buffer_offset).cast::<*const *const u8>()).byte_add(stride * agent_idx + field_offset);
/// ```
pub fn calc_turtle_var_offset(mir: &mir::Program, var: TurtleVarDesc) -> (usize, usize, usize) {
    fn stride_and_field_offset(
        turtle_schema: &TurtleSchema,
        field: AgentFieldDescriptor,
    ) -> (usize, usize) {
        // TODO(wishlist) it's inefficient to calculate the schemas every time.
        // see if we can cache this calculation as well as use it for making the
        // workspace
        let schemas: [Option<RowSchema>; 4] = turtle_schema.make_row_schemas();
        let row_schema = schemas[usize::from(field.buffer_idx)].as_ref().unwrap();
        let field_offset = row_schema.field(usize::from(field.field_idx)).offset;
        let stride = row_schema.stride();
        (stride, field_offset)
    }

    let turtle_schema = mir.turtle_schema.as_ref().unwrap();
    let (buffer_idx, stride, field_offset) = {
        let (field_desc, additional_offset) = turtle_schema.field_desc_and_offset(var);
        let (stride, field_offset) = stride_and_field_offset(turtle_schema, field_desc);
        (field_desc.buffer_idx, stride, field_offset + additional_offset)
    };
    let buffer_offset =
        offset_of!(Turtles, data) + (usize::from(buffer_idx) * size_of::<Option<RowBuffer>>());
    (buffer_offset, stride, field_offset)
}

pub fn turtle_var_type(schema: &TurtleSchema, var: TurtleVarDesc) -> ConcreteTy {
    match var {
        TurtleVarDesc::Who => NlFloat::CONCRETE_TY,
        TurtleVarDesc::Color => Color::CONCRETE_TY,
        TurtleVarDesc::Size => NlFloat::CONCRETE_TY,
        TurtleVarDesc::Pos => Point::CONCRETE_TY,
        TurtleVarDesc::Xcor => NlFloat::CONCRETE_TY,
        TurtleVarDesc::Ycor => NlFloat::CONCRETE_TY,
        TurtleVarDesc::Custom(field) => {
            let AgentSchemaField::Other(ty) = schema[schema.custom_fields()[field]] else {
                unreachable!("this is a custom field, so it cannot be part of the base data");
            };
            ty
        }
    }
}

// TODO(test) write tests for turtle initialization and access
