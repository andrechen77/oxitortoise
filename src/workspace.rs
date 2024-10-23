use crate::{procedure_prims::ProcedurePrims, world::World};

#[derive(Debug, Default)]
pub struct Workspace {
    pub procedure_prims: ProcedurePrims,
    pub world: World,
    pub updater: Updater,
    // TODO
    // rng
    // plot manager
}

#[derive(Debug, Default)]
pub struct Updater {
    // TODO
}

impl Workspace {
    pub fn new() -> Self {
        Workspace::default()
    }
}
