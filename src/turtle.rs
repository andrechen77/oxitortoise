use std::ops::DerefMut;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use std::fmt::{self, Debug};

use crate::{
    color::{self, Color},
    rng::NextInt,
};

/// A reference to a turtle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TurtleId(u64); // this is just the who number of the turtle

impl fmt::Display for TurtleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(turtle {})", self.0)
    }
}

#[derive(Debug)]
pub struct TurtleManager {
    /// The id to be given to the next turtle.
    next_id: TurtleId,
    breeds: HashMap<Rc<str>, Rc<RefCell<Breed>>>, // TODO why store by name? what if we just passed around an index?
    turtles_by_id: HashMap<TurtleId, Rc<RefCell<Turtle>>>,
    // updater
    next_int: Rc<RefCell<dyn NextInt>>,
}

impl TurtleManager {
    pub fn new(
        additional_breeds: impl IntoIterator<Item = Breed>,
        turtles_owns: Vec<Rc<str>>,
        links_owns: Vec<Rc<str>>,
        next_int: Rc<RefCell<dyn NextInt>>,
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
            .map(|breed| {
                (
                    breed.original_name.clone(),
                    Rc::new(RefCell::new(breed)),
                )
            })
            .collect();
        breeds.insert(
            Rc::from(BREED_NAME_TURTLES),
            Rc::new(RefCell::new(turtle_breed)),
        );
        breeds.insert(
            Rc::from(BREED_NAME_LINKS),
            Rc::new(RefCell::new(link_breed)),
        );
        TurtleManager {
            next_id: TurtleId(0),
            breeds,
            turtles_by_id: HashMap::new(),
            next_int,
        }
    }

    pub fn set_default_shape(&mut self, breed_name: &str, shape: Shape) {
        self.breeds
            .get_mut(breed_name)
            .unwrap() // TODO
            .borrow_mut()
            .shape = Some(Rc::new(shape));
    }

    pub fn get_turtle(&self, id: TurtleId) -> Option<Rc<RefCell<Turtle>>> {
        self.turtles_by_id.get(&id).cloned()
    }
}

impl TurtleManager {
    /// Creates the turtles and returns a list of their ids.
    pub fn create_turtles(
        &mut self,
        count: u64,
        breed_name: &str,
        xcor: f64,
        ycor: f64,
        mut on_create: impl FnMut(&Turtle),
    ) {
        let mut next_int = self.next_int.borrow_mut();

        let breed = self.breeds.get(breed_name).unwrap().clone(); // TODO deal with unwrap
        for _ in 0..count {
            let color = color::random_color(next_int.deref_mut());
            let heading = next_int.next_int(360) as f64;
            let id = self.next_id;
            self.next_id.0 += 1;
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
                id,
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
            on_create(&turtle);
            self.turtles_by_id.insert(id, Rc::new(RefCell::new(turtle)));
        }
    }
}

#[derive(Debug)]
pub struct Turtle {
    id: TurtleId,
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
    pub fn id(&self) -> TurtleId {
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
