use slotmap::SlotMap;

use super::shapes::Shapes;
use crate::sim::{
    agent_schema::{PatchSchema, TurtleSchema},
    // observer::Observer,
    patch::Patches,
    tick::Tick,
    topology::{Topology, TopologySpec},
    turtle::{Breed, BreedId, Turtles},
};

#[derive(Debug)]
pub struct World {
    // pub observer: RefCell<Observer>,
    pub turtles: Turtles,

    pub patches: Patches,

    pub topology: Topology,

    pub tick_counter: Tick,

    pub shapes: Shapes,
    // TODO add other fields
}

impl World {
    pub fn new(topology_spec: TopologySpec, turtle_breeds: SlotMap<BreedId, Breed>) -> Self {
        Self {
            // observer: RefCell::new(Observer::default()),
            turtles: Turtles::new(TurtleSchema::default(), turtle_breeds),
            patches: Patches::new(PatchSchema::default(), &topology_spec),
            topology: Topology::new(topology_spec),
            tick_counter: Tick::default(),
            shapes: Shapes::default(),
        }
    }

    pub fn clear_all(&mut self) {
        // self.observer.borrow_mut().clear_globals();
        self.patches.clear_patch_variables();
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
