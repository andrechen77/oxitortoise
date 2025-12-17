use std::mem::offset_of;

use crate::{mir, util::row_buffer::RowBuffer};

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalVarAddress {
    pub buffer_offset: usize,
    pub field_offset: usize,
}

pub fn calc_global_addr(program: &mir::Program, var: usize) -> GlobalVarAddress {
    let globals_schema = program.globals_schema.as_ref().unwrap();
    let row_schema = globals_schema.row_schema();
    GlobalVarAddress {
        buffer_offset: offset_of!(Globals, data),
        field_offset: row_schema.field(var).offset,
    }
}

#[derive(Debug)]
pub struct Globals {
    data: RowBuffer,
}
