use pretty_print::PrettyPrinter;

use std::{alloc::Layout, fmt::Write, mem::offset_of};

use super::shapes::Shapes;
use crate::{
    mir,
    sim::{
        observer::{Globals, GlobalsSchema},
        patch::{PatchSchema, Patches},
        tick::Tick,
        topology::Topology,
        turtle::{TurtleSchema, Turtles},
        value::NlFloat,
    },
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
        self.patches.clear_patch_variables(self.topology.spec());
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

    /// Derives a `Globals` from a `World`.
    pub fn mir_project_globals(world: mir::Place) -> mir::Place {
        world.proj_field(offset_of!(World, globals))
    }

    /// Derives a `Turtles` from a `World`.
    pub fn mir_project_turtles(world: mir::Place) -> mir::Place {
        world.proj_field(offset_of!(World, turtles))
    }

    /// Derives a `Patches` from a `World`.
    pub fn mir_project_patches(world: mir::Place) -> mir::Place {
        world.proj_field(offset_of!(World, patches))
    }

    /// Derives a `Topology` from a `World`.
    pub fn mir_project_topology(world: mir::Place) -> mir::Place {
        world.proj_field(offset_of!(World, topology))
    }

    pub fn mir_type_from_schemas(
        globals_schema: &GlobalsSchema,
        turtle_schema: &TurtleSchema,
        patch_schema: &PatchSchema,
    ) -> mir::MirType {
        let globals_ty = Globals::mir_type_from_schema(globals_schema);
        let turtles_ty = Turtles::mir_type_from_schema(turtle_schema);
        let patches_ty = Patches::mir_type_from_schema(patch_schema);

        mir::MirType::new_struct(
            Layout::new::<Self>(),
            vec![
                (offset_of!(Self, globals), globals_ty),
                (offset_of!(Self, turtles), turtles_ty),
                (offset_of!(Self, patches), patches_ty),
            ],
        )
    }
}

impl World {
    #[rustfmt::skip]
    pub fn generate_js_update_full(&self) -> String {
        let mut s = String::new();
        let mut p = PrettyPrinter::new(&mut s);

        let first_time = true;

        let _ = p.add_struct("", |p| {
            p.add_field_with("world", |p| p.add_struct("", |p| {
                p.add_field_with("0", |p| p.add_struct("", |p| {
                    p.add_field("WHO", 0)?;

                    if first_time {
                        p.add_field("patchesAllBlack", false)?;
                        p.add_field("patchesWithLabels", 0)?;
                        p.add_field("unbreededLinksAreDirected", false)?;
                        p.add_field("turtleBreeds", ["TURTLES"])?;
                    }

                    p.add_field("worldHeight", self.topology.world_height())?;
                    p.add_field("worldWidth", self.topology.world_width())?;
                    p.add_field("wrappingAllowedInX", self.topology.wrap_x())?;
                    p.add_field("wrappingAllowedInY", self.topology.wrap_y())?;
                    p.add_field("MINPXCOR", self.topology.min_pxcor())?;
                    p.add_field("MAXPXCOR", self.topology.max_pxcor())?;
                    p.add_field("MINPYCOR", self.topology.min_pycor())?;
                    p.add_field("MAXPYCOR", self.topology.max_pycor())?;
                    p.add_field("patchSize", 7.0)?;
                    p.add_field("ticks", self.tick_counter.get().map(NlFloat::get).unwrap_or(-1.0))?;
                    Ok(())
                }))
            }))?;
            p.add_field_with("observer", |p| p.add_struct("", |p| {
                if first_time {
                    p.add_field_with("targetAgent", |p| write!(p, "null"))?;
                    p.add_field("perspective", 0)?;
                }
                Ok(())
            }))?;
            p.add_field("drawingEvents", [] as [&str; 0])?;
            p.add_field_with("patches", |p| p.add_map(
                self.patches.patch_ids().map(|id| (id, ())),
                |p, id| write!(p, "{:?}", id.0),
                |p, (id, _)| p.add_struct("", |p| {
                    p.add_field("WHO", id.0)?;
                    p.add_field("PCOLOR", self.patches.get_patch_pcolor(id).expect("patch id from iter should be valid").get())?;
                    let base = self.patches.get_patch_base_data(id).expect("patch id from iter should be valid");
                    p.add_field("PLABEL", &base.plabel)?;
                    p.add_field("\"PLABEL-COLOR\"", base.plabel_color.get())?;
                    if first_time {
                        p.add_field("PXCOR", base.position.x)?;
                        p.add_field("PYCOR", base.position.y)?;
                    }
                    Ok(())
                })
            ))?;
            // doesn't handle dead turtles
            p.add_field_with("turtles", |p| p.add_map(
                self.turtles.turtle_ids().map(|id| (id, ())),
                |p, id| write!(p, "{:.0}", self.turtles.get_turtle_base_data(id).expect("turtle id from iter should be valid").who.get()),
                |p, (id, _)| p.add_struct("", |p| {
                    let base = self.turtles.get_turtle_base_data(id).expect("turtle id from iter should be valid");
                    let heading = self.turtles.get_turtle_heading(id).expect("turtle id from iter should be valid");
                    let position = self.turtles.get_turtle_position(id).expect("turtle id from iter should be valid");
                    p.add_field_with("WHO", |p| write!(p, "{:.0}", base.who.get()))?;
                    p.add_field_with("BREED", |p| write!(p, "\"{}\"", self.turtles.breeds()[&base.breed].name))?;
                    p.add_field_with("COLOR", |p| write!(p, "{}", base.color.get()))?;
                    p.add_field_with("HEADING", |p| write!(p, "{}", heading.to_float().get()))?;
                    p.add_field_with("\"LABEL-COLOR\"", |p| write!(p, "{}", base.label_color.get()))?;
                    p.add_field_with("\"HIDDEN?\"", |p| write!(p, "{}", base.hidden))?;
                    p.add_field_with("LABEL", |p| write!(p, "{:?}", base.label))?;
                    p.add_field_with("SHAPE", |p| write!(p, "{:?}", base.shape_name))?;
                    p.add_field_with("SIZE", |p| write!(p, "{}", base.size.get()))?;
                    p.add_field_with("XCOR", |p| write!(p, "{}", position.x))?;
                    p.add_field_with("YCOR", |p| write!(p, "{}", position.y))?;
                    Ok(())
                }),
            ))
        });

        s
    }
}
