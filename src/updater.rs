use std::{fmt::Write, rc::Rc};

use flagset::{flags, FlagSet};
use slotmap::SecondaryMap;

use crate::sim::{
    agent::AgentPosition as _, color::Color, patch::{Patch, PatchId}, tick::Tick, topology::{Heading, Point, TopologySpec}, turtle::{Turtle, TurtleId}, value::{Float, String}, world::World
};

pub trait WriteUpdate {
    /// Records in the updater that the specified properities of the world have
    /// changed to their new values.
    fn update_world_settings(
        &mut self,
        world: &World,
        properties_to_update: FlagSet<WorldSettingsProp>,
    );

    /// Records in the updater that the tick counter has been updated to the
    /// specified value.
    fn update_tick(&mut self, tick: Tick);

    // TODO updater the observer's perspective and target agent

    /// Records in the updater that the specified properties of a turtle have
    /// changed to their new values. If this is called on a turtle that the
    /// updater hasn't seen before, the updater also records that the turtle has
    /// been created.
    fn update_turtle(&mut self, turtle: &Turtle, properties_to_update: FlagSet<TurtleProp>);

    /// Records in the updater that the specified properties of a patch have
    /// changed to their new values.
    ///
    /// # Panics
    ///
    /// Panics if this updates a patch that this updater doesn't know about yet.
    /// Use [`WriteUpdate::update_world_settings`] to inform this updater about
    /// changes to the topology of the world that affect the number of patches.
    fn update_patch(&mut self, patch: &Patch, properties_to_update: FlagSet<PatchProp>);
}

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
        /// The presence of this flag means that the turtle has died.
        Death,
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
    pub shape_name: Option<&'static str>,
    pub size: Option<Float>,
    pub position: Option<Point>,
}

#[derive(Debug)]
pub struct UpdateAggregator {
    world: Option<WorldSettingsUpdate>,
    tick: Option<Tick>,
    /// Maps a TurtleId to the properties of that turtle that have changed.
    /// This may contain an entry for a dead turtle, if that turtle's slot has
    /// not been overwritten by another turtle that replaced it.
    ///
    /// This reason that we don't remove entries for dead turtles is so that the
    /// information that a turtle is dead is persistent in this field. That way,
    /// if a turtle is updated to be dead, and then some other update for that
    /// turtle is received, this aggregator can still know that the turtle is
    /// dead.
    // TODO is this behavior of remembering that dead turtles are dead correct?
    // there is a comment in the original Tortoise about turtles being reborn
    // but I don't understand it. Maybe it has to do with the fact that for that
    // implementation, the turtle who numbers are reused? This would not be a
    // concern here because TurtleIds are not reused even when who numbering
    // resets.
    // https://github.com/NetLogo/Tortoise/blob/8824b1da9db6f83d1a05d086928809efad6fc6b0/engine/src/main/coffee/engine/updater.coffee#L123
    turtles: SecondaryMap<TurtleId, TurtleUpdate>,
    /// Contains all turtles that have died in this upcoming update.
    dead_turtles: Vec<TurtleId>,
    /// Maps a PatchId to the properties of that patch that have changed.
    /// This data structure should maintain the same capacity when updates are
    /// collected. It should resize as necessary to accomodate new patches, and
    /// shrink if opportune when the world topology is known.
    patches: Vec<Option<PatchUpdate>>,
}

impl UpdateAggregator {
    pub fn new() -> Self {
        Self {
            world: None,
            tick: None,
            turtles: SecondaryMap::new(),
            dead_turtles: Vec::new(),
            patches: Vec::new(),
        }
    }
}

impl Default for UpdateAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl WriteUpdate for UpdateAggregator {
    fn update_world_settings(&mut self, world: &World, props: FlagSet<WorldSettingsProp>) {
        let world_update = self.world.get_or_insert_default();
        if props.contains(WorldSettingsProp::Topology) {
            let new_topology = *world.topology.spec();

            // update the topology itself in the update
            world_update.topology = Some(new_topology);

            // include slots for all the new patches to put their updates
            let mut next_patch_id = self.patches.len();
            self.patches.resize_with(new_topology.num_patches(), || {
                // record all the properties of new patch since this is its
                // first update
                let patch_data = world.patches[PatchId(next_patch_id)].data.borrow();
                next_patch_id += 1;
                Some(PatchUpdate {
                    pcolor: Some(patch_data.pcolor),
                    plabel: Some(patch_data.plabel.clone()),
                    plabel_color: Some(patch_data.plabel_color),
                })
            });
        }
        if props.contains(WorldSettingsProp::PatchSize) {
            world_update.patch_size = Some(7.0); // TODO update the patch size
        }
    }

    fn update_tick(&mut self, tick: Tick) {
        self.tick = Some(tick);
    }

    fn update_patch(&mut self, patch: &Patch, properties_to_update: FlagSet<PatchProp>) {
        let patch_data = patch.data.borrow();
        let patch_update = self.patches[patch.id().0].get_or_insert_with(Default::default);

        if properties_to_update.contains(PatchProp::Pcolor) {
            patch_update.pcolor = Some(patch_data.pcolor);
        }
        if properties_to_update.contains(PatchProp::Plabel) {
            patch_update.plabel = Some(patch_data.plabel.clone());
        }
        if properties_to_update.contains(PatchProp::PlabelColor) {
            patch_update.plabel_color = Some(patch_data.plabel_color);
        }
    }

    fn update_turtle(&mut self, turtle: &Turtle, properties_to_update: FlagSet<TurtleProp>) {
        if properties_to_update.contains(TurtleProp::Death) {
            self.dead_turtles.push(turtle.id());
            self.turtles.insert(turtle.id(), TurtleUpdate::Dead);
            return;
        }

        let Some(turtle_update_entry) = self.turtles.entry(turtle.id()) else {
            // we are receiving updates for a turtle whose spot has been taken
            // by a later update, so our turtle must be dead, so just ignore the
            // update.
            return;
        };
        let turtle_data = turtle.data.borrow();

        use slotmap::secondary::Entry;
        match turtle_update_entry {
            Entry::Occupied(e) => {
                // we have seen this turtle before, so only include the
                // requested properties
                if let TurtleUpdate::Alive(turtle_update) = e.into_mut() {
                    if properties_to_update.contains(TurtleProp::Breed) {
                        turtle_update.breed_name =
                            Some(turtle_data.breed.borrow().original_name.clone());
                    }
                    if properties_to_update.contains(TurtleProp::Color) {
                        turtle_update.color = Some(turtle_data.color);
                    }
                    if properties_to_update.contains(TurtleProp::Heading) {
                        turtle_update.heading = Some(turtle_data.heading);
                    }
                    if properties_to_update.contains(TurtleProp::LabelColor) {
                        turtle_update.label_color = Some(turtle_data.label_color);
                    }
                    if properties_to_update.contains(TurtleProp::Hidden) {
                        turtle_update.hidden = Some(turtle_data.hidden);
                    }
                    if properties_to_update.contains(TurtleProp::Label) {
                        turtle_update.label = Some(turtle_data.label.clone());
                    }
                    // TODO add pensize and penmode
                    if properties_to_update.contains(TurtleProp::Shape) {
                        turtle_update.shape_name = Some(turtle_data.shape.name);
                    }
                    if properties_to_update.contains(TurtleProp::Size) {
                        turtle_update.size = Some(turtle_data.size);
                    }
                    if properties_to_update.contains(TurtleProp::Position) {
                        turtle_update.position = Some(turtle_data.position);
                    }
                }
            }
            Entry::Vacant(e) => {
                // this is the first time we're seeing this turtle, so include
                // all properties that should be updated on turtle creation
                let turtle_update = AliveTurtleUpdate {
                    breed_name: Some(turtle_data.breed.borrow().original_name.clone()),
                    color: Some(turtle_data.color),
                    heading: Some(turtle_data.heading),
                    label_color: Some(turtle_data.label_color),
                    label: Some(turtle_data.label.clone()),
                    hidden: Some(turtle_data.hidden),
                    pen_mode_and_size: Some((false, 1.0)), // TODO add pensize and penmode
                    shape_name: Some(turtle_data.shape.name),
                    size: Some(turtle_data.size),
                    position: Some(turtle_data.position),
                };
                e.insert(TurtleUpdate::Alive(turtle_update));
            }
        }
    }
}

impl UpdateAggregator {
    /// Clears the contents of this aggregator after serializing them to a JS
    /// notation that can be consumed. This takes a world argument to do some last-minute lookup of things that I thought should not have been included in the update aggregation, but that we needed in order to get the correct serialized output:
    /// - the who numbers of turtles given their TurtleId
    /// - the patch coordinates of patches given their PatchId
    pub fn to_js(
        &mut self,
        mut w: impl Write,
        world: &World,
        first_time: bool,
    ) -> Result<(), std::fmt::Error> {
        // TODO actually dynamically calculate the update contents that are
        // first_time (doesn't apply to patch coords)

        write!(w, "{{ ")?;

        write!(w, "world: {{ 0: {{ ")?;
        write!(w, "WHO: 0, ")?;
        if first_time {
            write!(
                w,
                "patchesAllBlack: false, patchesWithLabels: 0, unbreededLinksAreDirected: false, "
            )?;

            // TODO what is up with the whole breeds thing? it's not listed
            // as a thing in updater.coffee but I still have to include turtle
            // and link breeds for them to work correctly.
            write!(w, "turtleBreeds: [\"TURTLES\"], ")?;
        }
        if let Some(world_update) = self.world.take() {
            if let Some(topology) = world_update.topology {
                write!(
                    w,
                    "worldHeight: {}, worldWidth: {}, wrappingAllowedInX: {}, wrappingAllowedInY: {}, MINPXCOR: {}, MAXPXCOR: {}, MINPYCOR: {}, MAXPYCOR: {}, ",
                    topology.patches_height, topology.patches_width, topology.wrap_x, topology.wrap_y, topology.min_pxcor, topology.max_pxcor(), topology.min_pycor(), topology.max_pycor,
                )?;
            }
            if let Some(patch_size) = world_update.patch_size {
                write!(w, "patchSize: {}, ", patch_size)?;
            }
        }
        if let Some(tick) = self.tick.take() {
            write!(w, "ticks: {}, ", tick.get().map(Float::get).unwrap_or(-1.0))?;
        }
        write!(w, "}} }}, ")?;

        write!(w, "observer: {{ ")?;
        if first_time {
            write!(w, "perspective: 0, targetAgent: null, ")?;
        }
        write!(w, "}}, ")?;

        write!(w, "drawingEvents: [],")?;

        write!(w, "patches: {{ ")?;
        for (patch_id, patch_update) in self
            .patches
            .iter_mut()
            // clear the updates from the aggregator as we take them
            .map(|p| p.take())
            .enumerate()
            // ignore patches without updates and convert index to PatchId
            .filter_map(|(i, p)| Some((PatchId(i), p?)))
        {
            write!(w, "{}: {{ ", patch_id.0)?;
            write!(w, "WHO: {}, ", patch_id.0)?;
            if let Some(pcolor) = patch_update.pcolor {
                write!(w, "PCOLOR: {}, ", pcolor.to_float().get())?;
            }
            if let Some(plabel) = patch_update.plabel {
                write!(w, "PLABEL: \"{}\", ", plabel)?;
            }
            if let Some(plabel_color) = patch_update.plabel_color {
                write!(w, "\"PLABEL-COLOR\": {}, ", plabel_color.to_float().get())?;
            }
            if first_time {
                let pos = world.patches[patch_id].position();
                write!(w, "PXCOR: {}, PYCOR: {}, ", pos.x, pos.y)?;
            }
            write!(w, "}}, ")?;
        }
        write!(w, "}}, ")?;

        write!(w, "turtles: {{ ")?;
        // TODO handle one turtle dying and then another turtle coming alive
        // and taking its who number in the same update
        self.dead_turtles.sort();
        self.dead_turtles.dedup();
        for turtle_id in self.dead_turtles.drain(..) {
            // look up the who number of the turtle as a pure integer. the
            // lookup should always succeed if we haven't missed any updates for
            // the world that was passed in, but fallback to -1 in case of bugs
            let who = world
                .turtles
                .get_turtle(turtle_id)
                .map(|t| t.who().0 as i64)
                .unwrap_or(-1);
            write!(w, "{}: {{ WHO: -1 }}, ", who)?;
        }
        for (turtle_id, turtle_update) in self.turtles.drain() {
            // only output alive turtle updates here; dead turtles were
            // already serialized above.
            if let TurtleUpdate::Alive(alive_update) = turtle_update {
                // look up the who number of the turtle as a pure integer. the
                // lookup should always succeed if we haven't missed any updates for
                // the world that was passed in, but fallback to -1 in case of bugs
                let who = world
                    .turtles
                    .get_turtle(turtle_id)
                    .map(|t| t.who().0 as i64)
                    .unwrap_or(-1);

                write!(w, "{}: {{ ", who)?;
                write!(w, "WHO: {}, ", who)?;
                if let Some(breed_name) = alive_update.breed_name {
                    write!(w, "BREED: \"{}\", ", breed_name)?;
                }
                if let Some(color) = alive_update.color {
                    write!(w, "COLOR: {}, ", color.to_float().get())?;
                }
                if let Some(heading) = alive_update.heading {
                    write!(w, "HEADING: {}, ", heading.to_float().get())?;
                }
                if let Some(label_color) = alive_update.label_color {
                    write!(w, "\"LABEL-COLOR\": {}, ", label_color.to_float().get())?;
                }
                if let Some(hidden) = alive_update.hidden {
                    write!(w, "\"HIDDEN?\": {}, ", hidden)?;
                }
                if let Some(label) = alive_update.label {
                    write!(w, "LABEL: \"{}\", ", label)?;
                }
                if let Some((pen_mode, pen_size)) = alive_update.pen_mode_and_size {
                    write!(
                        w,
                        "\"PEN-MODE\": \"{}\", \"PEN-SIZE\": {}, ",
                        if pen_mode { "down" } else { "up " },
                        pen_size
                    )?;
                }
                if let Some(shape_name) = alive_update.shape_name {
                    write!(w, "SHAPE: \"{}\", ", shape_name)?;
                }
                if let Some(size) = alive_update.size {
                    write!(w, "SIZE: {}, ", size.get())?;
                }
                if let Some(position) = alive_update.position {
                    write!(w, "XCOR: {}, YCOR: {}, ", position.x, position.y)?;
                }
                write!(w, "}}, ")?;
            }
        }
        write!(w, "}} ")?;

        write!(w, "}}")?;
        Ok(())
    }
}

pub type CanonUpdater = UpdateAggregator;
