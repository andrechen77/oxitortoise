use std::{cell::RefCell, rc::Rc};

use crate::{sim::world::World, util::rng::CanonRng};

#[derive(Debug)]
#[repr(C)]
pub struct Workspace {
    pub world: World,
    pub rng: Rc<RefCell<CanonRng>>,
}
