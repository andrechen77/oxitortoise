use std::cell::Cell;
use std::fmt::{self, Debug};
use std::rc::Weak;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use slotmap::SlotMap;

use crate::{
    sim::{
        agent_variables::{CustomAgentVariables, VarIndex, VariableMapper},
        color::{self, Color},
        topology::Point,
        value,
        world::World,
    },
    util::rng::NextInt,
};

use super::agent::{AgentIndexIntoWorld, AgentPosition};

/// The who number of a turtle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TurtleWho(u64);

impl TurtleWho {
    pub const INITIAL: TurtleWho = TurtleWho(0);
}

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
    type Output<'w> = Rc<RefCell<Turtle>>;

    fn index_into_world(self, world: &World) -> Option<Self::Output<'static>> {
        world.turtles.get_by_index(self)
    }
}

#[derive(Debug)]
pub struct Turtles {
    /// A back-reference to the world that includes this turtle manager.
    pub world: Weak<RefCell<World>>,
    /// The who number to be given to the next turtle.
    next_who: Cell<TurtleWho>,
    // TODO do we need to store the breeds in a refcell?
    // TODO why store by name? what if we just passed around an index?
    breeds: HashMap<Rc<str>, Rc<RefCell<Breed>>>,
    who_map: RefCell<HashMap<TurtleWho, TurtleId>>,
    // TODO consider not using an Rc, to allow for indexing to return a direct
    // &RefCell<Turtle> instead. this would also prevent from having to ensure
    // at runtime that the turtles are not borrowed if the `Turtles` is
    // exclusively borrowed. however, this would have safety concerns if we used
    // Box, which asserts uniqueness over its contents
    turtle_storage: RefCell<SlotMap<TurtleId, Rc<RefCell<Turtle>>>>,
}

impl Turtles {
    pub fn new(additional_breeds: impl IntoIterator<Item = Breed>) -> Self {
        let turtle_breed = Breed {
            original_name: Rc::from("turtles"),
            original_name_singular: Rc::from("turtle"),
            shape: Some(Rc::new(TURTLE_DEFAULT_SHAPE)),
            variable_mapper: VariableMapper::new(),
        };
        let link_breed = Breed {
            original_name: Rc::from("links"),
            original_name_singular: Rc::from("link"),
            shape: Some(Rc::new(LINK_DEFAULT_SHAPE)),
            variable_mapper: VariableMapper::new(),
        };
        let mut breeds: HashMap<_, _> = additional_breeds
            .into_iter()
            .map(|breed| (breed.original_name.clone(), Rc::new(RefCell::new(breed))))
            .collect();
        breeds.insert(
            Rc::from(BREED_NAME_TURTLES),
            Rc::new(RefCell::new(turtle_breed)),
        );
        breeds.insert(
            Rc::from(BREED_NAME_LINKS),
            Rc::new(RefCell::new(link_breed)),
        );

        Turtles {
            world: Weak::new(),
            next_who: Cell::new(TurtleWho::INITIAL),
            breeds,
            who_map: RefCell::new(HashMap::new()),
            turtle_storage: RefCell::new(SlotMap::with_key()),
        }
    }

    pub fn get_breed(&self, breed_name: &str) -> Option<Rc<RefCell<Breed>>> {
        self.breeds.get(breed_name).cloned()
    }

    pub fn get_by_index(&self, turtle_ref: TurtleId) -> Option<Rc<RefCell<Turtle>>> {
        self.turtle_storage.borrow().get(turtle_ref).cloned()
    }

    pub fn translate_who(&self, who: TurtleWho) -> Option<TurtleId> {
        self.who_map.borrow().get(&who).copied()
    }

    // # Safety
    //
    // This method mutably borrows from every included `RefCell<Turtle>`. You
    // must ensure at runtime that there are no borrows to any included turtle.
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
        for (_, turtle) in self.turtle_storage.get_mut().iter() {
            let mut turtle = turtle.borrow_mut();
            if let Some(new_to_old_idxs) = new_mappings.get(&Rc::as_ptr(&turtle.breed)) {
                turtle
                    .custom_variables
                    .set_variable_mapping(new_to_old_idxs);
            }
        }
    }

    /// Sets the backreferences of this structure and all structures owned by it
    /// to point to the specified world.
    pub fn set_world(&mut self, world: Weak<RefCell<World>>) {
        if !self.turtle_storage.get_mut().is_empty() {
            // This could be implemented by setting the world of the turtles
            // but there's no reason to implement it since the workspace should
            // be set before it is used.
            panic!("cannot set world when there are turtles");
        }
        self.world = world;
    }

    /// Creates the turtles and returns a list of their ids.
    pub fn create_turtles(
        &self,
        count: u64,
        breed_name: &str,
        spawn_point: Point,
        mut on_create: impl FnMut(TurtleId),
        next_int: &mut dyn NextInt,
    ) {
        let breed = self.breeds.get(breed_name).unwrap().clone(); // TODO deal with unwrap
        for _ in 0..count {
            let color = color::random_color(next_int);
            let heading = next_int.next_int(360) as f64;
            let who = self.next_who.get();
            self.next_who.set(TurtleWho(who.0 + 1));
            let shape = breed.borrow().shape.clone().unwrap_or_else(|| {
                self.breeds
                    .get(BREED_NAME_TURTLES)
                    .expect("default turtle breed should exist")
                    .borrow()
                    .shape
                    .as_ref()
                    .expect("default turtle breed should have a shape")
                    .clone()
            });
            let turtle = Turtle {
                world: self.world.clone(),
                who,
                breed: breed.clone(),
                color,
                heading,
                position: spawn_point,
                label: String::new(),
                label_color: color, // TODO use a default label color
                hidden: false,
                size: value::Float::new(1.0),
                shape,
                custom_variables: CustomAgentVariables::new(),
            };
            let turtle_ref = self
                .turtle_storage
                .borrow_mut()
                .insert(Rc::new(RefCell::new(turtle)));
            self.who_map.borrow_mut().insert(who, turtle_ref);
            on_create(turtle_ref);
        }
    }

    pub fn clear_turtles(&self) {
        self.turtle_storage.borrow_mut().clear();
        self.who_map.borrow_mut().clear();
        self.next_who.set(TurtleWho::INITIAL);
    }
}

#[derive(Debug)]
pub struct Turtle {
    /// A back-reference to the world that includes this turtle.
    world: Weak<RefCell<World>>,
    who: TurtleWho,
    breed: Rc<RefCell<Breed>>,
    /// The shape of this turtle due to its breed. This may or may not be the
    /// default shape of the turtle's breed.
    shape: Rc<Shape>,
    // name
    // linkmanager
    color: Color,
    heading: f64,
    position: Point,
    label: String, // TODO consider using the netlogo version of string for this
    label_color: Color,
    hidden: bool,
    size: value::Float,
    custom_variables: CustomAgentVariables,
}

impl Turtle {
    pub fn who(&self) -> TurtleWho {
        self.who
    }

    pub fn set_size(&mut self, size: value::Float) {
        self.size = size;
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn breed(&self) -> Rc<RefCell<Breed>> {
        self.breed.clone()
    }

    pub fn shape(&self) -> Rc<Shape> {
        self.shape.clone()
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn heading(&self) -> f64 {
        self.heading
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn label_color(&self) -> Color {
        self.label_color
    }

    pub fn hidden(&self) -> bool {
        self.hidden
    }

    pub fn size(&self) -> value::Float {
        self.size
    }

    pub fn get_custom(&self, index: VarIndex) -> &value::PolyValue {
        &self.custom_variables[index]
    }

    pub fn set_custom(&mut self, index: VarIndex, value: value::PolyValue) {
        self.custom_variables[index] = value;
    }

    pub fn get_world(&self) -> Rc<RefCell<World>> {
        self.world
            .upgrade()
            .expect("turtle's world should have been set")
    }
}

impl AgentPosition for Turtle {
    fn position(&self) -> Point {
        self.position
    }
}

#[derive(Debug)]
pub struct Breed {
    original_name: Rc<str>,
    #[allow(dead_code)]
    original_name_singular: Rc<str>,
    variable_mapper: VariableMapper<Turtle>,
    /// The default shape of this breed. `None` means that this breed should
    /// use the same shape as the default breed's shape. This must not be `None`
    /// if it is a default breed.
    shape: Option<Rc<Shape>>,
}

impl Breed {
    pub fn set_default_shape(&mut self, shape: Rc<Shape>) {
        self.shape = Some(shape);
    }
}

pub const BREED_NAME_TURTLES: &str = "TURTLES";
pub const BREED_NAME_LINKS: &str = "LINKS";

pub const TURTLE_DEFAULT_SHAPE: Shape = Shape {};
pub const LINK_DEFAULT_SHAPE: Shape = Shape {};

#[derive(Debug)]
pub struct Shape {
    // TODO
}
