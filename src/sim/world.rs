use std::{
    iter,
    rc::{Rc, Weak},
};

use crate::{
    sim::{
        observer::Observer,
        patch::Patches,
        tick::Tick,
        topology::{Topology, TopologySpec},
        turtle::Turtles,
    }, util::cell::RefCell, workspace::Workspace
};

#[derive(Debug)]
pub struct World {
    /// A back-reference to the workspace that includes this world.
    pub workspace: Weak<RefCell<Workspace>>,

    pub observer: RefCell<Observer>,
    pub turtles: Turtles,

    pub patches: Patches,

    pub topology: Topology,

    pub tick_counter: Tick,
    // TODO add other fields
}

impl World {
    pub fn new(topology_spec: TopologySpec) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            workspace: Weak::new(),
            observer: RefCell::new(Observer::default()),
            turtles: Turtles::new(iter::empty()),
            patches: Patches::new(&topology_spec),
            topology: Topology::new(topology_spec),
            tick_counter: Tick::default(),
        }))
    }

    /// Sets the backreferences of this structure and all structures owned by it
    /// to point to the specified workspace.
    pub fn set_workspace(&mut self, workspace: Weak<RefCell<Workspace>>) {
        self.workspace = workspace;
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
