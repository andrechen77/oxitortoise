use std::iter;

use crate::{
    sim::{
        observer::Observer,
        patch::Patches,
        tick::Tick,
        topology::{Topology, TopologySpec},
        turtle::Turtles,
    },
    util::cell::RefCell,
};

#[derive(Debug)]
pub struct World {
    pub observer: RefCell<Observer>,
    pub turtles: Turtles,

    pub patches: Patches,

    pub topology: Topology,

    pub tick_counter: Tick,
    // TODO add other fields
}

impl World {
    pub fn new(topology_spec: TopologySpec) -> Self {
        Self {
            observer: RefCell::new(Observer::default()),
            turtles: Turtles::new(iter::empty()),
            patches: Patches::new(&topology_spec),
            topology: Topology::new(topology_spec),
            tick_counter: Tick::default(),
        }
    }

    pub fn clear_all(&self) {
        self.observer.borrow_mut().clear_globals();
        self.patches.clear_all_patches();
        self.turtles.clear();
        /*
        @observer.resetPerspective()
        @clearLinks()
        @_declarePatchesAllBlack()
        @_resetPatchLabelCount()
        @ticker.clear()
        @_plotManager.clearAllPlots()
        @_outputClear()
        @clearDrawing()
        # Depending on global state for `Extensions` is not great, but Extensions depends on the workspace
        # and the workspace makes the world when it is created.  -Jeremy B July 19th
        Object.keys(Extensions).forEach( (extensionName) ->
            Extensions[extensionName].clearAll?()
        )
        return
         */ // TODO
    }
}
