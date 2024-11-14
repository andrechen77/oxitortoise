use std::fmt::{self, Debug};
use std::rc::Weak;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use slotmap::SlotMap;

use crate::color::{self, Color};
use crate::rng::NextInt;
use crate::world::World;

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

#[derive(Debug)]
pub struct Turtles {
    /// A back-reference to the world that includes this turtle manager.
    pub world: Weak<RefCell<World>>,
    /// The who number to be given to the next turtle.
    next_who: TurtleWho,
    // TODO do we need to store the breeds in a refcell?
    // TODO why store by name? what if we just passed around an index?
    breeds: HashMap<Rc<str>, Rc<RefCell<Breed>>>,
    who_map: HashMap<TurtleWho, TurtleId>,
    turtle_storage: SlotMap<TurtleId, Turtle>,
}

impl Turtles {
    pub fn new(
        additional_breeds: impl IntoIterator<Item = Breed>,
        turtles_owns: Vec<Rc<str>>,
        links_owns: Vec<Rc<str>>,
    ) -> Self {
        let turtle_breed = Breed {
            original_name: Rc::from("turtles"),
            original_name_singular: Rc::from("turtle"),
            variable_names: turtles_owns,
            shape: Some(Rc::new(TURTLE_DEFAULT_SHAPE)),
        };
        let link_breed = Breed {
            original_name: Rc::from("links"),
            original_name_singular: Rc::from("link"),
            variable_names: links_owns,
            shape: Some(Rc::new(LINK_DEFAULT_SHAPE)),
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
            next_who: TurtleWho::INITIAL,
            breeds,
            who_map: HashMap::new(),
            turtle_storage: SlotMap::with_key(),
        }
    }

    pub fn set_default_shape(&mut self, breed_name: &str, shape: Shape) {
        self.breeds
            .get_mut(breed_name)
            .unwrap() // TODO
            .borrow_mut()
            .shape = Some(Rc::new(shape));
    }

    pub fn get_by_index(&self, turtle_ref: TurtleId) -> Option<&Turtle> {
        self.turtle_storage.get(turtle_ref)
    }

    pub fn get_mut_by_index(&mut self, turtle_ref: TurtleId) -> Option<&mut Turtle> {
        self.turtle_storage.get_mut(turtle_ref)
    }

    pub fn translate_who(&self, who: TurtleWho) -> Option<TurtleId> {
        self.who_map.get(&who).copied()
    }

    /// Sets the backreferences of this structure and all structures owned by it
    /// to point to the specified world.
    pub fn set_world(&mut self, world: Weak<RefCell<World>>) {
        if !self.turtle_storage.is_empty() {
            // This could be implemented by setting the world of the turtles
            // but there's no reason to implement it since the workspace should
            // be set before it is used.
            panic!("cannot set world when there are turtles");
        }
        self.world = world;
    }

    /// Creates the turtles and returns a list of their ids.
    pub fn create_turtles(
        &mut self,
        count: u64,
        breed_name: &str,
        xcor: f64,
        ycor: f64,
        mut on_create: impl FnMut(TurtleId),
        next_int: &mut dyn NextInt,
    ) {
        let breed = self.breeds.get(breed_name).unwrap().clone(); // TODO deal with unwrap
        for _ in 0..count {
            let color = color::random_color(next_int);
            let heading = next_int.next_int(360) as f64;
            let who = self.next_who;
            self.next_who.0 += 1;
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
                id: who,
                breed: breed.clone(),
                color,
                heading,
                xcor,
                ycor,
                label: String::new(),
                label_color: color, // TODO use a default label color
                hidden: false,
                size: 1.0,
                shape,
            };
            let turtle_ref = self.turtle_storage.insert(turtle);
            self.who_map.insert(who, turtle_ref);
            on_create(turtle_ref);
        }
    }

    pub fn clear_turtles(&mut self) {
        self.turtle_storage.clear();
        self.who_map.clear();
        self.next_who = TurtleWho::INITIAL;
    }
}

#[derive(Debug)]
pub struct Turtle {
    /// A back-reference to the world that includes this turtle.
    world: Weak<RefCell<World>>,
    id: TurtleWho,
    breed: Rc<RefCell<Breed>>,
    /// The shape of this turtle due to its breed. This may or may not be the
    /// default shape of the turtle's breed.
    shape: Rc<Shape>,
    // name
    // updateVarsByName
    // varmanager
    // linkmanager
    color: Color,
    heading: f64,
    xcor: f64,
    ycor: f64,
    label: String,
    label_color: Color,
    hidden: bool,
    size: f64,
}

impl Turtle {
    pub fn id(&self) -> TurtleWho {
        self.id
    }

    pub fn set_size(&mut self, size: f64) {
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

    pub fn xcor(&self) -> f64 {
        self.xcor
    }

    pub fn ycor(&self) -> f64 {
        self.ycor
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

    pub fn size(&self) -> f64 {
        self.size
    }

    pub fn get_world(&self) -> Rc<RefCell<World>> {
        self.world
            .upgrade()
            .expect("turtle's world should have been set")
    }
}

#[derive(Debug)]
pub struct Breed {
    original_name: Rc<str>,
    #[allow(dead_code)]
    original_name_singular: Rc<str>,
    #[allow(dead_code)]
    variable_names: Vec<Rc<str>>,
    /// The default shape of this breed. `None` means that this breed should
    /// use the same shape as the default breed's shape. This must not be `None`
    /// if it is a default breed.
    shape: Option<Rc<Shape>>,
}

impl Breed {
    pub fn get_shape(&self) {}
}

pub const BREED_NAME_TURTLES: &str = "TURTLES";
pub const BREED_NAME_LINKS: &str = "LINKS";

pub const TURTLE_DEFAULT_SHAPE: Shape = Shape {};
pub const LINK_DEFAULT_SHAPE: Shape = Shape {};

#[derive(Debug)]
pub struct Shape {
    // TODO
}
