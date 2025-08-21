use std::{any::TypeId, ops::Index};

use crate::sim::patch::PatchBaseData;
use crate::{
    sim::{turtle::TurtleBaseData, value::NetlogoInternalType},
    util::row_buffer::{RowBuffer, RowSchema},
};

// TODO make better, actual documentation for how the agents are laid out
// hybrid SoA-AoS model: the set of fields for an agent is organized into
// a set of sets: {{A, B}, {C, D}, {E}, {F}} means that A and B are stored in
// an array of structs, C and D are stored in a separate array of structs, E
// is stored in a single array, and F is stored in a single array

// Some built-in agent variables are stored in the base data of an agent (the `A`
// generic parameter) and some are stored in other fields.

// Describes all the fields of an agent and how they are stored. The base data
// for an agent is always stored in the first buffer.

#[derive(Debug)]
pub struct TurtleSchema {
    position: AgentFieldDescriptor,
    heading: AgentFieldDescriptor,
    field_groups: Vec<AgentSchemaFieldGroup>,
}

impl TurtleSchema {
    pub fn new(
        heading_buffer_idx: u8,
        position_buffer_idx: u8,
        custom_fields: &[(NetlogoInternalType, u8)],
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
        heading_group.fields.push(AgentSchemaField::Other(NetlogoInternalType::HEADING));
        let position_group = &mut field_groups[position_buffer_idx as usize];
        let position_field_idx = position_group.fields.len() as u8;
        position_group.fields.push(AgentSchemaField::Other(NetlogoInternalType::POINT));

        // add custom fields
        for (field_type, buffer_idx) in custom_fields {
            field_groups[*buffer_idx as usize]
                .fields
                .push(AgentSchemaField::Other(field_type.clone()));
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
        }
    }

    pub fn make_row_buffers<const N: usize>(&self) -> [Option<RowBuffer>; N] {
        make_row_buffers_impl::<TurtleBaseData, N>(&self.field_groups)
    }

    pub fn heading(&self) -> AgentFieldDescriptor {
        self.heading
    }

    pub fn position(&self) -> AgentFieldDescriptor {
        self.position
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

#[derive(Debug)]
pub struct PatchSchema {
    pcolor: AgentFieldDescriptor,
    field_groups: Vec<AgentSchemaFieldGroup>,
    custom_fields: Vec<AgentFieldDescriptor>,
}

impl PatchSchema {
    pub fn new(
        pcolor_buffer_idx: u8,
        custom_fields: &[(NetlogoInternalType, u8)],
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
            pcolor_buffer_idx.max(custom_fields.iter().map(|(_, idx)| *idx).max().unwrap_or(0));
        while field_groups.len() <= max_buffer_idx as usize {
            field_groups.push(AgentSchemaFieldGroup {
                avoid_occupancy_bitfield: false,
                fields: Vec::new(),
            });
        }

        // add pcolor field
        field_groups[pcolor_buffer_idx as usize]
            .fields
            .push(AgentSchemaField::Other(NetlogoInternalType::COLOR));

        // add custom fields and collect their descriptors
        let mut custom_field_descriptors = Vec::new();
        for (field_type, buffer_idx) in custom_fields {
            let field_idx = field_groups[*buffer_idx as usize].fields.len() as u8;
            field_groups[*buffer_idx as usize]
                .fields
                .push(AgentSchemaField::Other(field_type.clone()));
            custom_field_descriptors
                .push(AgentFieldDescriptor { buffer_idx: *buffer_idx, field_idx });
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

    pub fn make_row_buffers<const N: usize>(&self) -> [Option<RowBuffer>; N] {
        make_row_buffers_impl::<PatchBaseData, N>(&self.field_groups)
    }

    pub fn pcolor(&self) -> AgentFieldDescriptor {
        self.pcolor
    }

    pub fn custom_fields(&self) -> impl Iterator<Item = &AgentFieldDescriptor> {
        self.custom_fields.iter()
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

#[derive(Debug)]
pub struct AgentSchemaFieldGroup {
    /// Whether the fields in this group should have an occupancy bitfield
    /// indicating their presence. Indicating `true` subjects the fields to
    /// additional constraints according to the RowBuffer documentation, but
    /// also saves space for fields that would pack better without a bitfield
    /// between each row.
    pub avoid_occupancy_bitfield: bool,
    pub fields: Vec<AgentSchemaField>,
}

#[derive(Debug)]
pub enum AgentSchemaField {
    /// The "field" which holds base data as built-in agent variables. This
    /// is always the first field in the first row buffer.
    BaseData,
    /// A variable stored anywhere other than the first field of the first
    /// buffer.
    Other(NetlogoInternalType),
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
}

fn make_row_buffers_impl<A: 'static, const N: usize>(
    field_groups: &[AgentSchemaFieldGroup],
) -> [Option<RowBuffer>; N] {
    let AgentSchemaField::BaseData = field_groups[0].fields[0] else {
        panic!("The first field in the first buffer must be the base data.");
    };

    let row_buffers: [Option<RowBuffer>; N] = std::array::from_fn(|buffer_idx| {
        let Some(buffer_fields) = field_groups.get(buffer_idx) else {
            return None;
        };

        // the types and sizes of the fields in this buffer
        let mut field_types = Vec::new();
        for (field_idx, buffer_field) in buffer_fields.fields.iter().enumerate() {
            let type_id = match buffer_field {
                AgentSchemaField::BaseData => {
                    if (buffer_idx, field_idx) == (0, 0) {
                        TypeId::of::<A>()
                    } else {
                        panic!("Base data can only be the first field in the first buffer.");
                    }
                }
                AgentSchemaField::Other(r#type) => r#type.into(),
            };
            field_types.push(type_id);
        }

        Some(RowBuffer::new(RowSchema::new(&field_types, !buffer_fields.avoid_occupancy_bitfield)))
    });

    row_buffers
}
