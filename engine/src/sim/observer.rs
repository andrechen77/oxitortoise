use std::{
    collections::HashMap,
    fmt::{self, Debug, Write},
    mem::offset_of,
    rc::Rc,
};

use either::Either;
use pretty_print::PrettyPrinter;

use crate::{
    mir,
    sim::value::{NlBool, NlFloat, NlList, NlString, PackedAny},
    util::{
        reflection::{ConcreteTy, Reflect},
        row_buffer::{RowBuffer, RowSchema},
    },
};

pub struct Globals {
    schema: GlobalsSchema,
    pub data: RowBuffer,
    fallback_fields: HashMap<usize, PackedAny>,
}

impl Globals {
    pub fn new(schema: GlobalsSchema) -> Self {
        let mut data = RowBuffer::new_with_capacity(schema.make_row_schema(), 1);
        let mut fallback_fields = HashMap::new();

        // initialize variables to zero
        for (var_id, (_, ty)) in schema.custom_fields.iter().enumerate() {
            if ty.info().is_zeroable {
                data.row_mut(0).insert_zeroable(var_id);
            } else {
                fallback_fields.insert(var_id, PackedAny::ZERO);
            }
        }

        Self { schema, data, fallback_fields }
    }

    pub fn get<T: Reflect>(&self, var_index: usize) -> Either<&T, &PackedAny> {
        if let Some(field) = self.data.row(0).get(var_index) {
            Either::Left(field)
        } else {
            Either::Right(&self.fallback_fields[&var_index])
        }
    }
}

impl fmt::Debug for Globals {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut p = PrettyPrinter::new(f);
        p.add_struct("Globals", |p| {
            p.add_field("globals_schema", |p| write!(p, "{:?}", self.schema))?;
            p.add_field("globals", |p| {
                p.add_map(
                    self.schema
                        .custom_fields
                        .iter()
                        .enumerate()
                        .map(|(i, (name, ty))| (&**name, (i, *ty))),
                    |p, name| write!(p, "{:?}", name),
                    |p, (_, (i, ty))| {
                        fn print_field<T: Reflect + Debug>(
                            p: &mut PrettyPrinter<impl Write>,
                            globals: &Globals,
                            var_index: usize,
                        ) -> fmt::Result {
                            match globals.get::<T>(var_index) {
                                Either::Left(field) => write!(p, "{:?}", field),
                                Either::Right(field) => write!(p, "fallback {:?}", field),
                            }
                        }
                        if ty == NlFloat::CONCRETE_TY {
                            print_field::<NlFloat>(p, self, i)
                        } else if ty == NlBool::CONCRETE_TY {
                            print_field::<NlBool>(p, self, i)
                        } else if ty == NlString::CONCRETE_TY {
                            print_field::<NlString>(p, self, i)
                        } else if ty == NlList::CONCRETE_TY {
                            print_field::<NlList>(p, self, i)
                        } else {
                            write!(p, "unknown type {:?}", ty)
                        }
                    },
                )
            })
        })
    }
}

#[derive(Debug, Clone)]
pub struct GlobalsSchema {
    custom_fields: Vec<(Rc<str>, ConcreteTy)>,
}

impl GlobalsSchema {
    pub fn new(custom_fields: &[(&Rc<str>, ConcreteTy)]) -> Self {
        Self {
            custom_fields: custom_fields.iter().map(|(name, ty)| (Rc::clone(name), *ty)).collect(),
        }
    }

    pub fn make_row_schema(&self) -> RowSchema {
        RowSchema::new(&self.custom_fields.iter().map(|(_, ty)| *ty).collect::<Vec<_>>(), true)
    }
}

/// Similar to [`calc_turtle_var_offset`], but excluding the returned stride
/// value (since there is only one instance of each global variable and
/// therefore only one row).
pub fn calc_global_var_offset(program: &mir::Program, var: usize) -> (usize, usize) {
    let row_schema = program.globals_schema.as_ref().unwrap().make_row_schema();

    (offset_of!(Globals, data), row_schema.field(var).offset)
}
