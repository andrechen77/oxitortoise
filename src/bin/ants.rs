use std::rc::Rc;

use oxitortoise::procedure_prims::Command;
use oxitortoise::turtle_manager::{Shape, BREED_NAME_TURTLES};
use oxitortoise::workspace::Workspace;

// define the Ants model. this is a direct translation of this code
// https://github.com/NetLogo/Tortoise/blob/master/resources/test/dumps/Ants.js
pub fn main() {
    let mut workspace = Workspace::new();
    let Workspace {
        procedure_prims,
        world,
        ..
    } = &mut workspace;
    procedure_prims.define_command(Command {
        name: Rc::from("setup"),
        start: 637,
        end: 715,
        action: Box::new(|workspace| {
            workspace.world.clear_all();
        }),
    });
    world
        .turtle_manager
        .set_default_shape(BREED_NAME_TURTLES, Shape {});
    world.turtle_manager.create_turtles(
        world
            .observer
            .get_global("population")
            .expect("compiler assumes this variable exists")
            .try_into()
            .expect("compiler assumes this is a number"),
        BREED_NAME_TURTLES,
        0.0,
        0.0,
    );

    unimplemented!();
}
