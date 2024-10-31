use std::{cell::RefCell, rc::Rc};

use crate::{execution::ProcedureManager, rng::{NextInt, RandIntGenerator}, world::World};

#[derive(Debug)]
pub struct Workspace {
    pub procedures: ProcedureManager<U>,
    pub world: Rc<RefCell<World>>,
    pub rng: Rc<RefCell<dyn NextInt>>,
    // TODO
    // plot manager
}

impl<U> Workspace<U> {
    pub fn new() -> Rc<RefCell<Self>> {
        // create the structure first without worrying about backreferences
        let rng = Rc::new(RefCell::new(RandIntGenerator::new()));
        let workspace = Rc::new(RefCell::new(Self {
            procedures: ProcedureManager::new(),
            world: World::new(),
            rng,
        }));

        // now we must set all back-references to have a consistent data
        // structure
        workspace.borrow_mut().world.borrow_mut().set_workspace(Rc::downgrade(&workspace));

        workspace
    }
}
