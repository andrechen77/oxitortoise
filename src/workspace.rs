use std::rc::Rc;

use crate::{
    sim::{topology::TopologySpec, world::World},
    util::{cell::RefCell, rng::CanonRng},
};

#[derive(Debug)]
pub struct Workspace {
    pub world: Rc<RefCell<World>>,
    pub rng: Rc<RefCell<CanonRng>>,
    // TODO add other fields
    // plot manager
}

impl Workspace {
    pub fn new(topology: TopologySpec) -> Rc<RefCell<Self>> {
        // create the structure first without worrying about backreferences
        let rng = Rc::new(RefCell::new(CanonRng::new(0))); // TODO use a better seed
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
