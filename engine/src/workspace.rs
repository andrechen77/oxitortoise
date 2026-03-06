use std::sync::{Arc, Mutex};

use crate::{sim::world::World, util::rng::CanonRng};

#[derive(Debug)]
#[repr(C)]
pub struct Workspace {
    pub world: World,
    pub rng: Arc<Mutex<CanonRng>>,
}
