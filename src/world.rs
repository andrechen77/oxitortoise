use std::iter;

use crate::{observer::Observer, turtle_manager::TurtleManager};

#[derive(Debug)]
pub struct World {
    // self manager
    // breed manager
    pub link_manager: LinkManager,
    pub observer: Observer,
    pub self_manager: SelfManager,
    pub ticker: Ticker,
    pub topology: Topology,
    pub turtle_manager: TurtleManager,
    pub patch_manager: PatchManager,
    // TODO
}

impl Default for World {
    fn default() -> Self {
        World {
            link_manager: LinkManager::default(),
            observer: Observer::default(),
            self_manager: SelfManager::default(),
            ticker: Ticker::default(),
            topology: Topology::default(),
            turtle_manager: TurtleManager::new(iter::empty(), vec![], vec![]),
            patch_manager: PatchManager::default(),
        }
    }
}

impl World {
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
}

#[derive(Debug, Default)]
pub struct LinkManager {}
#[derive(Debug, Default)]
pub struct SelfManager {}
#[derive(Debug, Default)]
pub struct Ticker {}
#[derive(Debug, Default)]
pub struct Topology {}
#[derive(Debug, Default)]
pub struct PatchManager {}
