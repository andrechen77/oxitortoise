use std::rc::Rc;

use crate::{
    sim::{topology::TopologySpec, world::World},
    util::{cell::RefCell, rng::CanonRng},
};

#[derive(Debug)]
pub struct Workspace {
    pub world: World,
    pub rng: Rc<RefCell<CanonRng>>,
    // TODO add other fields
    // plot manager
}
