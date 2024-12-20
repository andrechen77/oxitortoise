use std::rc::Rc;

use flagset::{flags, FlagSet};
use slotmap::SecondaryMap;

use crate::sim::{
    color::Color,
    patch::PatchId,
    tick::Tick,
    topology::{Heading, Point, TopologySpec},
    turtle::TurtleId,
    value::Float,
    world::World,
};

// TODO is there a better way to send the updates without having to create
// a reference to the agent being updated? for example, we could just save the
// agent id and OR all the changed flags together to find all the properties
// that were changed

pub trait WriteUpdate {
    /// Records in the updater that the specified properties of a turtle have
    /// changed to their new values. If this is called on a turtle that the
    /// updater hasn't seen before, the updater also records that the turtle has
    /// been created.
    fn update_turtle(&mut self, turtle: TurtleId, properties_to_update: FlagSet<TurtleProp>);

    /// Records in the updater that the specified properties of a patch have
    /// changed to their new values.
    fn update_patch(&mut self, patch: PatchId, properties_to_update: FlagSet<PatchProp>);

    // TODO updater the observer's perspective and target agent

    /// Records in the updater that the specified properities of the world have
    /// changed to their new values.
    fn update_world_properties(&mut self, properties_to_update: FlagSet<WorldProp>);

    /// Gets all the updates recorded in this updater since the last time this
    /// method was called.
    ///
    /// This method should not modify the world.
    fn get_update(&mut self, world: &World) -> UpdateContent;
}

#[derive(Debug, Default)]
pub struct UpdateContent {
    world: Option<WorldUpdate>,
    patches: Vec<(PatchId, PatchUpdate)>,
    turtles: Vec<(TurtleId, TurtleUpdate)>,
}

flags! {
    pub enum WorldProp: u16 {
        /// Represents the topology of the world, including its dimensions,
        /// boundary points, and wrapping behavior.
        Topology,
        PatchSize,
        Ticks,
    }
}

#[derive(Debug, Default)]
pub struct WorldUpdate {
    pub topology: Option<TopologySpec>,
    pub patch_size: Option<f64>,
    pub ticks: Option<Tick>,
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
    pub hidden: Option<bool>,
    pub shape_name: Option<&'static str>,
    pub size: Option<Float>,
    pub position: Option<Point>,
}

#[derive(Debug)]
pub struct UpdateAggregator {
    world: FlagSet<WorldProp>,
    /// Maps a TurtleId to the properties of that turtle that need to be
    /// updated. This may contain an entry for a dead turtle, if that turtle's
    /// slot has not been overwritten by another turtle that replaced it.
    ///
    /// This reason that we don't remove entries for dead turtles is so that the
    /// information that a turtle is dead is persistent in this field. That way,
    /// if a turtle is updated to be dead, and then some other update for that
    /// turtle is received, this aggregator can still know that the turtle is
    /// dead.
    // TODO is this behavior of remembering that dead turtles are dead correct?
    // there is a comment in the original Tortoise about turtles being reborn
    // but I don't understand it.
    // https://github.com/NetLogo/Tortoise/blob/8824b1da9db6f83d1a05d086928809efad6fc6b0/engine/src/main/coffee/engine/updater.coffee#L123
    turtles: SecondaryMap<TurtleId, FlagSet<TurtleProp>>,
    /// Contains all dead turtles.
    dead_turtles: Vec<TurtleId>,
    /// Maps a PatchId to the properties of that patch that need to be updated.
    /// This data structure should maintain the same capacity when updates are
    /// collected. It should resize as necessary to accomodate new patches, and
    /// shrink if opportune when the world topology is known.
    patches: Vec<FlagSet<PatchProp>>,
}

impl UpdateAggregator {
    pub fn new() -> Self {
        Self {
            world: FlagSet::default(),
            turtles: SecondaryMap::new(),
            dead_turtles: Vec::new(),
            patches: Vec::new(),
        }
    }
}

impl WriteUpdate for UpdateAggregator {
    fn update_world_properties(&mut self, properties_to_update: FlagSet<WorldProp>) {
        self.world |= properties_to_update;
    }

    fn update_patch(&mut self, patch: PatchId, properties_to_update: FlagSet<PatchProp>) {
        if patch.0 >= self.patches.len() {
            self.patches.resize_with(patch.0 + 1, Default::default);
        }
        self.patches[patch.0] |= properties_to_update;
    }

    fn update_turtle(&mut self, turtle: TurtleId, properties_to_update: FlagSet<TurtleProp>) {
        if properties_to_update.contains(TurtleProp::Death) {
            self.dead_turtles.push(turtle);
        } else if let Some(props_entry) = self.turtles.entry(turtle) {
            *props_entry.or_default() |= properties_to_update;
        }
    }

    fn get_update(&mut self, world: &World) -> UpdateContent {
        let mut update = UpdateContent::default();

        // collect the world update content
        if !self.world.is_empty() {
            let world_props = self.world;
            self.world.clear();

            let world_update = update.world.get_or_insert_default();
            if world_props.contains(WorldProp::Topology) {
                world_update.topology = Some(*world.topology.spec());
            }
            if world_props.contains(WorldProp::PatchSize) {
                // TODO update the patch size
            }
            if world_props.contains(WorldProp::Ticks) {
                world_update.ticks = Some(world.tick_counter.clone());
            }
        }

        // collect the patch update content
        for (patch_id, patch_props) in self
            .patches
            .iter_mut()
            .enumerate()
            .map(|(grid_index, props)| (PatchId(grid_index), props))
        {
            if patch_props.is_empty() {
                continue;
            }

            let mut patch_update = PatchUpdate::default();
            let patch_data = world.patches[patch_id].data.borrow();
            if patch_props.contains(PatchProp::Pcolor) {
                patch_update.pcolor = Some(patch_data.pcolor);
            }
            if patch_props.contains(PatchProp::Plabel) {
                patch_update.plabel = Some(patch_data.plabel.clone());
            }
            if patch_props.contains(PatchProp::PlabelColor) {
                patch_update.plabel_color = Some(patch_data.plabel_color);
            }
            update.patches.push((patch_id, patch_update));

            patch_props.clear();
        }

        // collect the turtle update content
        for turtle_id in self.dead_turtles.drain(..) {
            update.turtles.push((turtle_id, TurtleUpdate::Dead));
        }
        // draining the slotmap is okay because the slotmap keeps its underlying
        // storage capacity
        for (turtle_id, turtle_props) in self.turtles.drain() {
            if turtle_props.is_empty() {
                continue;
            }

            if turtle_props.contains(TurtleProp::Death) {
                update.turtles.push((turtle_id, TurtleUpdate::Dead));
                continue;
            }

            let mut turtle_update = AliveTurtleUpdate::default();
            let turtle_data = world.turtles.get_turtle(turtle_id).unwrap().data.borrow();
            if turtle_props.contains(TurtleProp::Breed) {
                turtle_update.breed_name = Some(turtle_data.breed.borrow().original_name.clone());
            }
            if turtle_props.contains(TurtleProp::Color) {
                turtle_update.color = Some(turtle_data.color);
            }
            if turtle_props.contains(TurtleProp::Heading) {
                turtle_update.heading = Some(turtle_data.heading);
            }
            if turtle_props.contains(TurtleProp::LabelColor) {
                turtle_update.label_color = Some(turtle_data.label_color);
            }
            if turtle_props.contains(TurtleProp::Hidden) {
                turtle_update.hidden = Some(turtle_data.hidden);
            }
            // TODO add pensize and penmode
            if turtle_props.contains(TurtleProp::Shape) {
                turtle_update.shape_name = Some(turtle_data.shape.name);
            }
            if turtle_props.contains(TurtleProp::Size) {
                turtle_update.size = Some(turtle_data.size);
            }
            if turtle_props.contains(TurtleProp::Position) {
                turtle_update.position = Some(turtle_data.position);
            }

            update
                .turtles
                .push((turtle_id, TurtleUpdate::Alive(turtle_update)));
        }

        update
    }
}
