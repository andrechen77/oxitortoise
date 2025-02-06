use std::fmt::{self, Debug};
use std::{collections::HashMap, rc::Rc};

use crate::util::cell::RefCell;
use crate::{
    sim::{
        agent_variables::{CustomAgentVariables, VarIndex, VariableMapper},
        color::Color,
        topology::Point,
        value,
        world::World,
    },
    util::rng::Rng,
};

use super::agent::{AgentIndexIntoWorld, AgentPosition};
use super::shapes::{ShapeId, Shapes};
use super::topology::Heading;

mod turtle_storage;

use turtle_storage::TurtleStorage;

/// The who number of a turtle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct TurtleWho(pub u64);

impl fmt::Display for TurtleWho {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(turtle {})", self.0)
    }
}

slotmap::new_key_type! {
    /// An invalidate-able reference to a turtle. This is implemented as a
    /// generational index into the [`Turtles`] data structure.
    pub struct TurtleId;
}

impl AgentIndexIntoWorld for TurtleId {
    type Output<'w> = &'w Turtle;

    fn index_into_world(self, world: &World) -> Option<Self::Output<'_>> {
        world.turtles.get_turtle(self)
    }
}

#[derive(Debug, Default)]
pub struct Turtles {
    // TODO do we need to store the breeds in a refcell?
    // TODO why store by name? what if we just passed around an index?
    breeds: HashMap<Rc<str>, Rc<RefCell<Breed>>>,
    /// Owns the data for the turtles.
    turtle_storage: TurtleStorage,
}

impl Turtles {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn breeds(&self) -> impl Iterator<Item = &Rc<RefCell<Breed>>> {
        self.breeds.values()
    }

    pub fn get_breed(&self, breed_name: &str) -> Option<Rc<RefCell<Breed>>> {
        self.breeds.get(breed_name).cloned()
    }

    pub fn get_turtle(&self, turtle_id: TurtleId) -> Option<&Turtle> {
        self.turtle_storage.get_turtle(turtle_id)
    }

    pub fn turtle_ids(&self) -> Vec<TurtleId> {
        self.turtle_storage.turtle_ids()
    }

    pub fn translate_who(&self, who: TurtleWho) -> Option<TurtleId> {
        self.turtle_storage.translate_who(who)
    }

    pub fn declare_custom_variables<'a>(
        &mut self,
        variables_by_breed: impl Iterator<Item = (&'a str, Vec<Rc<str>>)>,
    ) {
        // create a mapping from a changed breed to its new-to-old custom
        // indexes. store the breeds by their address instead of their contents.
        // this is not only faster, but ensures that in the degenerate case,
        // breeds with the same contents are treated as distinct (however note
        // that this case should never happen as long as breeds are stored by
        // their names)
        let mut new_mappings = HashMap::new();

        for (breed_name, new_custom_variables) in variables_by_breed {
            let breed = self.breeds.get(breed_name).expect("breed should exist");
            let new_to_old_custom_idxs = breed
                .borrow_mut()
                .variable_mapper
                .declare_custom_variables(new_custom_variables);
            new_mappings.insert(Rc::as_ptr(breed), new_to_old_custom_idxs);
        }

        // make sure all turtles have the correct mappings in their custom
        // variables
        for turtle in self.turtle_storage.iter_mut() {
            if let Some(new_to_old_idxs) =
                new_mappings.get(&Rc::as_ptr(&turtle.data.get_mut().breed))
            {
                turtle
                    .data
                    .get_mut()
                    .custom_variables
                    .set_variable_mapping(new_to_old_idxs);
            }
        }
    }

    /// Creates the turtles and returns a list of their ids.
    pub fn create_turtles(
        &self,
        count: u64,
        breed_name: &str,
        spawn_point: Point,
        mut on_create: impl FnMut(TurtleId),
        next_int: &mut dyn Rng,
        shapes: &Shapes,
    ) {
        let breed = self.breeds.get(breed_name).unwrap(); // TODO deal with unwrap
        for _ in 0..count {
            let color = Color::random(next_int);
            let heading = Heading::random(next_int);
            let shape = breed.borrow().shape.unwrap_or_else(|| {
                *self
                    .breeds
                    .get(BREED_NAME_TURTLES)
                    .expect("default turtle breed should exist")
                    .borrow()
                    .shape
                    .as_ref()
                    .expect("default turtle breed should have a shape")
            });
            let shape_name = shapes
                .get_shape(shape)
                .unwrap() // TODO deal with unwrap
                .name
                .clone();
            let turtle_data = TurtleData {
                breed: breed.clone(),
                color,
                heading,
                position: spawn_point,
                label: String::new(),
                label_color: color, // TODO use a default label color
                hidden: false,
                size: value::Float::new(1.0),
                shape_name,
                custom_variables: CustomAgentVariables::new(),
            };

            let turtle_id = self.turtle_storage.add_turtle(turtle_data);
            on_create(turtle_id);
        }
    }

    pub fn clear(&self) {
        self.turtle_storage.clear();
    }
}

#[derive(Debug)]
pub struct Turtle {
    id: TurtleId,
    who: TurtleWho,
    pub data: RefCell<TurtleData>,
}

impl Turtle {
    pub fn id(&self) -> TurtleId {
        self.id
    }

    pub fn who(&self) -> TurtleWho {
        self.who
    }
}

#[derive(Debug, Default, Clone)]
pub struct TurtleData {
    pub breed: Rc<RefCell<Breed>>,
    /// The shape of this turtle due to its breed. This may or may not be the
    /// default shape of the turtle's breed.
    pub shape_name: Rc<str>,
    pub color: Color,
    pub heading: Heading,
    pub position: Point,
    pub label: String, // TODO consider using the netlogo version of string for this
    pub label_color: Color,
    pub hidden: bool,
    pub size: value::Float,
    custom_variables: CustomAgentVariables,
}

impl TurtleData {
    pub fn get_custom(&self, index: VarIndex) -> &value::PolyValue {
        &self.custom_variables[index]
    }

    pub fn set_custom(&mut self, index: VarIndex, value: value::PolyValue) {
        self.custom_variables[index] = value;
    }
}

impl AgentPosition for Turtle {
    fn position(&self) -> Point {
        self.data.borrow().position
    }
}

#[derive(Debug, Default)]
pub struct Breed {
    pub original_name: Rc<str>,
    #[allow(dead_code)]
    pub original_name_singular: Rc<str>,
    variable_mapper: VariableMapper,
    /// The default shape of this breed. `None` means that this breed should
    /// use the same shape as the default breed's shape. This must not be `None`
    /// if it is a default breed.
    shape: Option<ShapeId>,
}

impl Breed {
    pub fn set_default_shape(&mut self, shape: ShapeId) {
        self.shape = Some(shape);
    }
}

pub const BREED_NAME_TURTLES: &str = "TURTLES";

pub const TURTLE_DEFAULT_SHAPE: &str = "default";
