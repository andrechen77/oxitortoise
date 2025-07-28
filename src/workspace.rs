use std::{mem::offset_of, rc::Rc};

use crate::{
    sim::world::World,
    util::{cell::RefCell, rng::CanonRng},
};

#[no_mangle]
static OFFSET_WORKSPACE_TO_WORLD: usize = offset_of!(Workspace, world);

#[derive(Debug)]
#[repr(C)]
pub struct Workspace {
    pub world: World,
    pub rng: Rc<RefCell<CanonRng>>,
    // TODO add other fields
    // plot manager
}
