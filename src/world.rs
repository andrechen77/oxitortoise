use std::{
    cell::RefCell,
    iter,
    rc::{Rc, Weak},
};

use crate::{
    agent::{Agent, AgentId, AgentMut}, observer::Observer, patch::Patches, tick::Tick, topology::Topology, turtle::Turtles, workspace::Workspace
};

#[derive(Debug)]
pub struct World {
    /// A back-reference to the workspace that includes this world.
    pub workspace: Weak<RefCell<Workspace>>,
    pub observer: Observer,
    pub turtles: Turtles,
    pub patches: Patches,
    pub topology: Topology,
    pub tick_counter: Tick,
    // TODO add other fields
}

impl World {
    pub fn new(topology: Topology) -> Rc<RefCell<Self>> {
        let world = Rc::new(RefCell::new(Self {
            workspace: Weak::new(),
            observer: Observer::default(),
            turtles: Turtles::new(iter::empty()),
            patches: Patches::new(&topology),
            topology,
            tick_counter: Tick::default(),
        }));

        // now we must set all back-references to have a consistent data
        // structure
        {
            let mut w = world.borrow_mut();
            w.observer.set_world(Rc::downgrade(&world));
            w.turtles.set_world(Rc::downgrade(&world));
        }

        world
    }

    /// Sets the backreferences of this structure and all structures owned by it
    /// to point to the specified workspace.
    pub fn set_workspace(&mut self, workspace: Weak<RefCell<Workspace>>) {
        self.workspace = workspace;
    }

    pub fn clear_all(&mut self) {
        self.observer.clear_globals();
        self.patches.clear_all_patches();
        self.turtles.clear_turtles();
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

    pub fn get_agent(&self, agent_ref: AgentId) -> Option<Agent<'_>> {
        match agent_ref {
            AgentId::Observer => Some(Agent::Observer(&self.observer)),
            AgentId::Turtle(turtle_ref) => {
                Some(Agent::Turtle(self.turtles.get_by_index(turtle_ref)?))
            }
            AgentId::Patch(_id) => todo!(),
            AgentId::Link(_id) => todo!(),
        }
    }

    pub fn get_agent_mut(&mut self, agent_ref: AgentId) -> Option<AgentMut<'_>> {
        match agent_ref {
            AgentId::Observer => Some(AgentMut::Observer(&mut self.observer)),
            AgentId::Turtle(turtle_ref) => {
                Some(AgentMut::Turtle(self.turtles.get_mut_by_index(turtle_ref)?))
            }
            AgentId::Patch(_id) => todo!(),
            AgentId::Link(_id) => todo!(),
        }
    }
}
