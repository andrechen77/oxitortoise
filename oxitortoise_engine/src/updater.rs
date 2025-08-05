use std::{mem::offset_of, rc::Rc};

use flagset::{flags, FlagSet};

use crate::sim::{
    color::Color,
    patch::PatchId,
    tick::Tick,
    topology::{Heading, Point, TopologySpec},
    turtle::TurtleWho,
    value::Float,
};

// TODO updater fields for the observer's perspective and target agent

flags! {
    pub enum WorldSettingsProp: u8 {
        /// Represents the topology of the world, including its dimensions,
        /// boundary points, and wrapping behavior.
        Topology,
        PatchSize,
    }
}

#[derive(Debug, Default)]
pub struct WorldSettingsUpdate {
    pub topology: Option<TopologySpec>,
    pub patch_size: Option<f64>,
}

flags! {
    // TODO what to do about the pxcor and pycor properties? it seems the
    // original tortoise engine outputs updates to them but it doesn't really
    // make sense for them to be updated
    pub enum PatchProp: u8 {
        Pcolor,
        Plabel,
        PlabelColor,
    }
}

#[derive(Debug, Default)]
pub struct PatchUpdate {
    pub pcolor: Option<Color>,
    pub plabel: Option<String>,
    pub plabel_color: Option<Color>,
}

flags! {
    pub enum TurtleProp: u16 {
        Breed,
        Color,
        Heading,
        LabelColor,
        Label,
        Hidden,
        PenSize,
        PenMode,
        Shape,
        Size,
        Position,
    }
}

#[derive(Debug)]
pub enum TurtleUpdate {
    Dead,
    Alive(AliveTurtleUpdate),
}

#[derive(Debug, Default)]
pub struct AliveTurtleUpdate {
    pub breed_name: Option<Rc<str>>,
    pub color: Option<Color>,
    pub heading: Option<Heading>,
    pub label_color: Option<Color>,
    pub label: Option<String>,
    pub pen_mode_and_size: Option<(bool, f64)>,
    pub hidden: Option<bool>,
    pub shape_name: Option<String>,
    pub size: Option<Float>,
    pub position: Option<Point>,
}

#[no_mangle]
static OFFSET_DIRTY_TO_TICK: usize = offset_of!(DirtyAggregator, tick);
#[no_mangle]
static OFFSET_DIRTY_TO_TURTLES: usize = offset_of!(DirtyAggregator, turtles_ffi);
#[no_mangle]
static OFFSET_DIRTY_TO_PATCHES: usize = offset_of!(DirtyAggregator, patches_ffi);

/// Tracks all the dirty state that needs to be included in the next update.
#[derive(Debug)]
pub struct DirtyAggregator {
    /// The current tick number.
    pub tick: Tick,
    pub world: FlagSet<WorldSettingsProp>,
    /// A raw pointer into the data buffer of the field
    /// [`DirtyAggregator::turtles`].
    ///
    /// While this is redundant, Rust leaves the layout of Vec unspecified, so
    /// we use this to allow foreign code to access the data buffer directly.
    /// Methods that modify the underlying Vec will keep this pointer in sync.
    ///
    /// There is no protection against moving the Vec around, so provenance must
    /// be re-established by accessing this address with the provenance of the
    /// actual vector (irrelevant at the machine level). As such, there is no
    /// safe abstraction to dereference this.
    turtles_ffi: *mut FlagSet<TurtleProp>,
    /// Maps a turtle's index to the properties of that turtle that have
    /// changed. If there is no live turtle with that index, the entry is all
    /// zeros. The client has the responsibility to ensure that the slice is
    /// large enough to accomodate all turtles.
    turtles: Vec<FlagSet<TurtleProp>>,
    /// A raw pointer into the data buffer of the field
    /// [`DirtyAggregator::patches`]. See the comment on
    /// [`DirtyAggregator::turtles_ffi`] for more details.
    patches_ffi: *mut FlagSet<PatchProp>,
    /// Maps a patch's ID to the properties of that patch that have changed. The
    /// client has the responsibility to ensure that the slice is large enough
    /// to accomodate all patches
    patches: Vec<FlagSet<PatchProp>>,
    /// Contains the who numbers of all the turtles that have died in the upcoming update.
    dead_turtles: Vec<TurtleWho>,
}

impl DirtyAggregator {
    pub fn new() -> Self {
        let mut turtles = Vec::new();
        let turtles_ffi = turtles.as_mut_ptr();
        let mut patches = Vec::new();
        let patches_ffi = patches.as_mut_ptr();

        Self {
            tick: Tick::new(),
            world: FlagSet::default(),
            turtles_ffi,
            patches_ffi,
            turtles,
            patches,
            dead_turtles: Vec::new(),
        }
    }

    pub fn reserve_turtles(&mut self, count: usize) {
        self.turtles.reserve(count);
        self.turtles_ffi = self.turtles.as_mut_ptr();
    }

    pub fn reserve_patches(&mut self, count: usize) {
        self.patches.reserve(count);
        self.patches_ffi = self.patches.as_mut_ptr();
    }

    pub fn get_turtles_mut(&mut self) -> &mut [FlagSet<TurtleProp>] {
        &mut self.turtles
    }
    pub fn get_patches_mut(&mut self) -> &mut [FlagSet<PatchProp>] {
        &mut self.patches
    }

    // TODO accessors for non-public fields
}

impl Default for DirtyAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct UpdateContent {
    pub world: Option<WorldSettingsUpdate>,
    pub tick: Tick,
    pub turtles: Vec<(TurtleWho, TurtleUpdate)>,
    pub dead_turtles: Vec<TurtleWho>,
    pub patches: Vec<(PatchId, PatchUpdate)>,
}

// TODO create UpdateContent by reading the world state. also have a function to
// serialize the update content to JSON. could consider using unchecked accesses
// or JIT-compiled function to grab the properties (to avoid constant type
// checks)
