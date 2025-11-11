use std::{collections::HashMap, rc::Rc};

use slotmap::SlotMap;

slotmap::new_key_type! {
    /// An invalidate-able reference to to a shape.
    pub struct ShapeId;
}

#[derive(Debug, Default)]
pub struct Shapes {
    #[allow(dead_code)] // remove when used
    name_map: HashMap<Rc<str>, ShapeId>,
    shapes: SlotMap<ShapeId, Shape>,
}

impl Shapes {
    pub fn get_shape(&self, id: ShapeId) -> Option<&Shape> {
        self.shapes.get(id)
    }
}

#[derive(Debug)]
pub struct Shape {
    pub name: Rc<str>,
}
