use crate::util::reflection::Reflect;
use crate::{util::reflection::ConcreteTy, util::row_buffer::RowSchema};

// TODO(mvp) make better, actual documentation for how the agents are laid out
// hybrid SoA-AoS model: the set of fields for an agent is organized into
// a set of sets: {{A, B}, {C, D}, {E}, {F}} means that A and B are stored in
// an array of structs, C and D are stored in a separate array of structs, E
// is stored in a single array, and F is stored in a single array

// Some built-in agent variables are stored in the base data of an agent (the `A`
// generic parameter) and some are stored in other fields.

// Describes all the fields of an agent and how they are stored. The base data
// for an agent is always stored in the first buffer.

#[derive(Clone, Debug)]
pub struct AgentSchemaFieldGroup {
    /// Whether the fields in this group should have an occupancy bitfield
    /// indicating their presence. Indicating `true` subjects the fields to
    /// additional constraints according to the RowBuffer documentation, but
    /// also saves space for fields that would pack better without a bitfield
    /// between each row.
    pub avoid_occupancy_bitfield: bool,
    pub fields: Vec<AgentSchemaField>,
}

#[derive(Clone, Debug)]
pub enum AgentSchemaField {
    /// The "field" which holds base data as built-in agent variables. This
    /// is always the first field in the first row buffer.
    BaseData,
    /// A variable stored anywhere other than the first field of the first
    /// buffer.
    Other(ConcreteTy),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct AgentFieldDescriptor {
    /// The index of the buffer that stores the data for this field.
    pub buffer_idx: u8,
    /// The index of the field within the buffer.
    pub field_idx: u8,
}

impl AgentFieldDescriptor {
    pub const BASE_DATA: Self = Self { buffer_idx: 0, field_idx: 0 };

    pub fn to_u16(&self) -> u16 {
        (self.field_idx as u16) << 8 | self.buffer_idx as u16
    }

    pub fn from_u16(value: u16) -> Self {
        Self { buffer_idx: (value & 0xff) as u8, field_idx: ((value >> 8) & 0xff) as u8 }
    }
}

pub(crate) fn make_row_schemas<A: Reflect, const N: usize>(
    field_groups: &[AgentSchemaFieldGroup],
) -> [Option<RowSchema>; N] {
    let AgentSchemaField::BaseData = field_groups[0].fields[0] else {
        panic!("The first field in the first buffer must be the base data.");
    };

    let row_schemas: [Option<RowSchema>; N] = std::array::from_fn(|buffer_idx| {
        let buffer_fields = field_groups.get(buffer_idx)?;

        // the types and sizes of the fields in this buffer
        let mut field_types = Vec::new();
        for (field_idx, buffer_field) in buffer_fields.fields.iter().enumerate() {
            let type_id = match buffer_field {
                AgentSchemaField::BaseData => {
                    if (buffer_idx, field_idx) == (0, 0) {
                        A::CONCRETE_TY
                    } else {
                        panic!("Base data can only be the first field in the first buffer.");
                    }
                }
                AgentSchemaField::Other(r#type) => *r#type,
            };
            field_types.push(type_id);
        }

        Some(RowSchema::new(&field_types, !buffer_fields.avoid_occupancy_bitfield))
    });

    row_schemas
}
