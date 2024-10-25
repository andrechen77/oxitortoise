use std::{cell::RefCell, rc::Rc};

use crate::{procedure::ProcedureManager, rng::{NextInt, RandIntGenerator}, world::World};

#[derive(Debug)]
pub struct Workspace {
    pub procedures: ProcedureManager,
    pub world: World,
    pub rng: Rc<RefCell<dyn NextInt>>,
    // TODO
    // plot manager
}

impl Workspace {
    pub fn new() -> Self {
        let rng = Rc::new(RefCell::new(RandIntGenerator::new()));
        Self {
            procedures: ProcedureManager::new(),
            world: World::new(rng.clone()),
            rng,
        }
    }
}
