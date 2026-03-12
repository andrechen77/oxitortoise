use std::{
    collections::HashMap,
    fmt::{self, Debug, Write},
    sync::Arc,
};

use either::Either;
use pretty_print::PrettyPrinter;

use crate::{
    sim::value::{NlBool, NlFloat, NlList, NlString, PackedAny},
    util::{
        reflection::{Reflect, Type},
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
            if ty.is_zeroable {
                data.row_mut(0).set_zero(var_id);
            } else {
                fallback_fields.insert(var_id, PackedAny::ZERO);
            }
        }

        Self { schema, data, fallback_fields }
    }

    pub fn get<T: Reflect + 'static>(&self, var_index: usize) -> Either<&T, &PackedAny> {
        if let Some(field) = self.data.row(0).get(var_index) {
            Either::Left(field)
        } else {
            Either::Right(&self.fallback_fields[&var_index])
        }
    }

    pub fn get_mut<T: Reflect + 'static>(
        &mut self,
        var_index: usize,
    ) -> Either<&mut T, &mut PackedAny> {
        if let Some(field) = self.data.row_mut(0).get_mut(var_index) {
            Either::Left(field)
        } else {
            Either::Right(
                self.fallback_fields
                    .get_mut(&var_index)
                    .expect("global variable should always exist"),
            )
        }
    }
}

impl fmt::Debug for Globals {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut p = PrettyPrinter::new(f);
        p.add_struct("Globals", |p| {
            p.add_field_with("globals_schema", |p| write!(p, "{:?}", self.schema))?;
            p.add_field_with("globals", |p| {
                p.add_map(
                    self.schema
                        .custom_fields
                        .iter()
                        .enumerate()
                        .map(|(i, (name, ty))| (&**name, (i, ty))),
                    |p, name| write!(p, "{:?}", name),
                    |p, (_, (i, ty))| {
                        fn print_field<T: Reflect + Debug + 'static>(
                            p: &mut PrettyPrinter<impl Write>,
                            globals: &Globals,
                            var_index: usize,
                        ) -> fmt::Result {
                            match globals.get::<T>(var_index) {
                                Either::Left(field) => write!(p, "{:?}", field),
                                Either::Right(field) => write!(p, "fallback {:?}", field),
                            }
                        }
                        if ty.is::<NlFloat>() {
                            print_field::<NlFloat>(p, self, i)
                        } else if ty.is::<NlBool>() {
                            print_field::<NlBool>(p, self, i)
                        } else if ty.is::<NlString>() {
                            print_field::<NlString>(p, self, i)
                        } else if ty.is::<NlList>() {
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
    custom_fields: Vec<(Arc<str>, Type)>,
}

impl GlobalsSchema {
    pub fn new(custom_fields: &[(&Arc<str>, Type)]) -> Self {
        Self {
            custom_fields: custom_fields.iter().map(|(name, ty)| (Arc::clone(name), *ty)).collect(),
        }
    }

    pub fn make_row_schema(&self) -> RowSchema {
        RowSchema::new(&self.custom_fields.iter().map(|(_, ty)| *ty).collect::<Vec<_>>(), true)
    }

    /// Calculates the offset of a field from the start of the globals data
    pub fn offset_of_field(&self, field_index: usize) -> usize {
        self.make_row_schema().field(field_index).offset
    }
}
