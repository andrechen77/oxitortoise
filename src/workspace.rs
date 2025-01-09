use crate::{
    sim::{topology::TopologySpec, world::World},
    util::{cell::RefCell, rng::CanonRng},
};

#[derive(Debug)]
pub struct Workspace {
    pub world: World,
    pub rng: RefCell<CanonRng>,
    // TODO add other fields
    // plot manager
}

impl Workspace {
    pub fn new(topology: TopologySpec) -> Self {
        let rng = RefCell::new(CanonRng::new(0)); // TODO use a better seed
        Self {
            world: World::new(topology),
            rng,
        }
    }
}
