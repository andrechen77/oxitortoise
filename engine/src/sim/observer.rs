use std::mem::offset_of;

use crate::{
    mir,
    util::{
        reflection::ConcreteTy,
        row_buffer::{RowBuffer, RowSchema},
    },
};

#[derive(Debug)]
pub struct Globals {
    pub data: RowBuffer,
}

impl Globals {
    pub fn new(schema: GlobalsSchema) -> Self {
        Self { data: RowBuffer::new(schema.row_schema) }
    }
}

#[derive(Debug, Clone)]
pub struct GlobalsSchema {
    row_schema: RowSchema,
}

impl GlobalsSchema {
    pub fn new(field_types: &[ConcreteTy]) -> Self {
        Self { row_schema: RowSchema::new(field_types, true) }
    }
}

/// Similar to [`calc_turtle_var_offset`], but excluding the returned stride
/// value (since there is only one instance of each global variable and
/// therefore only one row).
pub fn calc_global_var_offset(program: &mir::Program, var: usize) -> (usize, usize) {
    let globals_schema = program.globals_schema.as_ref().unwrap();

    (offset_of!(Globals, data), globals_schema.row_schema.field(var).offset)
}
