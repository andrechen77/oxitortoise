use std::{cell::RefCell, rc::Rc};

use crate::{
    sim::{
        topology::Topology,
        world::World,
    },
    rng::{NextInt, RandIntGenerator},
    execution::ProcedureManager,
};

#[derive(Debug)]
pub struct Workspace {
    pub procedures: ProcedureManager,
    pub world: Rc<RefCell<World>>,
    pub rng: Rc<RefCell<dyn NextInt>>,
    // TODO add other fields
    // plot manager
}

impl Workspace {
    pub fn new(topology: Topology) -> Rc<RefCell<Self>> {
        // create the structure first without worrying about backreferences
        let rng = Rc::new(RefCell::new(RandIntGenerator::new()));
        let workspace = Rc::new(RefCell::new(Self {
            procedures: ProcedureManager::new(),
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
