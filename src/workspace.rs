use std::{cell::RefCell, rc::Rc};

use crate::{
    sim::{topology::TopologySpec, world::World},
    util::rng::{NextInt, RandIntGenerator},
};

#[derive(Debug)]
pub struct Workspace {
    pub world: Rc<RefCell<World>>,
    pub rng: Rc<RefCell<dyn NextInt>>,
    // TODO add other fields
    // plot manager
}

impl Workspace {
    pub fn new(topology: TopologySpec) -> Rc<RefCell<Self>> {
        // create the structure first without worrying about backreferences
        let rng = Rc::new(RefCell::new(RandIntGenerator::new()));
        let workspace = Rc::new(RefCell::new(Self {
            world: World::new(topology),
            rng,
        }));

        // now we must set all back-references to have a consistent data
        // structure
        workspace
            .borrow_mut()
            .world
            .borrow_mut()
            .set_workspace(Rc::downgrade(&workspace));

        workspace
    }
}
