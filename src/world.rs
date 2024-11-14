use std::{
    cell::RefCell,
    iter,
    rc::{Rc, Weak},
};

use crate::{
    agent::Agent, agent_id::AgentId, observer::Observer, turtle::TurtleManager,
    workspace::Workspace,
};

#[derive(Debug)]
pub struct World {
    /// A back-reference to the workspace that includes this world.
    pub workspace: Weak<RefCell<Workspace>>,
    pub observer: Observer,
    pub turtle_manager: TurtleManager,
    // TODO
}

impl World {
    pub fn new() -> Rc<RefCell<Self>> {
        let world = Rc::new(RefCell::new(Self {
            workspace: Weak::new(),
            observer: Observer::default(),
            turtle_manager: TurtleManager::new(iter::empty(), vec![], vec![]),
        }));

        // now we must set all back-references to have a consistent data
        // structure
        let mut w = world.borrow_mut();
        w.observer.set_world(Rc::downgrade(&world));
        w.turtle_manager.set_world(Rc::downgrade(&world));

        todo!()
    }

    /// Sets the backreferences of this structure and all structures owned by it
    /// to point to the specified workspace.
    pub fn set_workspace(&mut self, workspace: Weak<RefCell<Workspace>>) {
        self.workspace = workspace;
    }

    pub fn clear_all(&mut self) {
        self.observer.clear_globals();
        /*
        @observer.resetPerspective()
        @turtleManager.clearTurtles()
        @clearPatches()
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

    pub fn get_agent(&self, id: AgentId) -> Option<Agent> {
        match id {
            AgentId::Observer => todo!(),
            AgentId::Turtle(id) => Some(Agent::Turtle(self.turtle_manager.get_turtle(id)?)),
            AgentId::Patch(_id) => todo!(),
            AgentId::Link(_id) => todo!(),
        }
    }
}
