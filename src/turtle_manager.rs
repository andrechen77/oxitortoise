use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    color::{self, Color},
    rng::NextInt,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TurtleId(u64);

#[derive(Debug)]
pub struct TurtleManager {
    /// The id to be given to the next turtle.
    next_id: TurtleId,
    breeds: HashMap<String, Rc<RefCell<Breed>>>, // TODO why store by name? what if we just passed around an index?
    turtles_by_id: HashMap<TurtleId, Rc<RefCell<Turtle>>>,
    // updater
    next_int: NextInt,
}

impl TurtleManager {
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
            .map(|breed| {
                (
                    breed.original_name.to_string(),
                    Rc::new(RefCell::new(breed)),
                )
            })
            .collect();
        breeds.insert(
            BREED_NAME_TURTLES.to_owned(),
            Rc::new(RefCell::new(turtle_breed)),
        );
        breeds.insert(
            BREED_NAME_LINKS.to_owned(),
            Rc::new(RefCell::new(link_breed)),
        );
        TurtleManager {
            next_id: TurtleId(0),
            breeds,
            turtles_by_id: HashMap::new(),
            next_int: NextInt {},
        }
    }

    pub fn set_default_shape(&mut self, breed_name: &str, shape: Shape) {
        self.breeds
            .get_mut(breed_name)
            .unwrap() // TODO
            .borrow_mut()
            .shape = Some(Rc::new(shape));
    }

    pub fn create_turtles(&mut self, count: u64, breed_name: &str, xcor: f64, ycor: f64) {
        let breed = self.breeds.get(breed_name).unwrap().clone(); // TODO unwrap
        for _ in 0..count {
            let color = color::random_color(&mut self.next_int);
            let heading = self.next_int.next(360) as f64;
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
                label_color: color, // TODO make this the default label color
                hidden: false,
                size: 1.0,
                shape,
            };
            self.turtles_by_id.insert(id, Rc::new(RefCell::new(turtle)));
        }
        todo!()
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

#[derive(Debug)]
pub struct Breed {
    original_name: Rc<str>,
    original_name_singular: Rc<str>,
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
