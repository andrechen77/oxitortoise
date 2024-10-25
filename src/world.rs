use std::{cell::RefCell, iter, rc::Rc};

use crate::{agent::Agent, agent_id::AgentId, observer::Observer, rng::NextInt, turtle::TurtleManager};

#[derive(Debug)]
pub struct World {
    pub observer: Observer,
    pub turtle_manager: TurtleManager,
    // TODO
}

impl World {
    pub fn new(rng: Rc<RefCell<dyn NextInt>>) -> Self {
        Self {
            observer: Observer::default(),
            turtle_manager: TurtleManager::new(iter::empty(), vec![], vec![], rng),
        }
    }

    pub fn clear_all(&mut self) {
        /*
        @observer.clearCodeGlobals()
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
