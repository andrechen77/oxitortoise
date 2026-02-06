use super::shapes::Shapes;
use crate::sim::{
    observer::Globals, patch::Patches, tick::Tick, topology::Topology, turtle::Turtles,
};

#[derive(Debug)]
pub struct World {
    pub globals: Globals,
    pub turtles: Turtles,
    pub patches: Patches,
    pub topology: Topology,
    pub tick_counter: Tick,
    pub shapes: Shapes,
}

impl World {
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
         */
        // TODO(mvp) finish clear_all implementation
    }
}
