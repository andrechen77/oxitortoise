use std::cell::RefCell;
use std::rc::Rc;

use flagset::FlagSet;
use oxitortoise::shuffle_iterator::ShuffleIterator;
use oxitortoise::topology::Topology;
use oxitortoise::turtle::{Shape, BREED_NAME_TURTLES};
use oxitortoise::updater::{PrintUpdate, TurtleProperty, Update};
use oxitortoise::value::Value;
use oxitortoise::workspace::Workspace;
use oxitortoise::{color, value};

// define the Ants model. this is a direct translation of this code
// https://github.com/NetLogo/Tortoise/blob/master/resources/test/dumps/Ants.js
fn run_ants_model() -> Rc<RefCell<Workspace>> {
    let w = Workspace::new(Topology {
        min_pxcor: -12,
        max_pycor: 12,
        world_width: 25,
        world_height: 25,
    });
    let mut workspace = w.borrow_mut();
    let mut world = workspace.world.borrow_mut();
    let mut updater = PrintUpdate;

    // `globals [ population ]`
    workspace
        .world
        .borrow_mut()
        .observer
        .create_widget_global(Rc::from("population"), Value::Float(value::Float(2.0)));

    // `clear-all`
    world.clear_all();

    // `set-default-shape turtles "bug"`
    world
        .turtles
        .set_default_shape(BREED_NAME_TURTLES, Shape {});

    // `create-turtles ...`
    let num = world
        .observer
        .get_global("population")
        .expect("compiler assumes this variable exists")
        .to_u64_round_to_zero()
        .expect("compiler assumes this is a number");
    let mut new_turtles = Vec::new();
    world.turtles.create_turtles(
        num,
        BREED_NAME_TURTLES,
        0.0,
        0.0,
        |turtle| new_turtles.push(turtle.clone()),
        &mut *workspace.rng.borrow_mut(),
    );

    for turtle in ShuffleIterator::new(&mut new_turtles, workspace.rng.clone()) {
        let turtle = world
            .turtles
            .get_mut_by_index(*turtle)
            .expect("turtle should exist because it was just created");

        // `set size 2`
        turtle.set_size(2.0);

        // `set color red`
        turtle.set_color(color::RED);

        updater.update_turtle(&*turtle, TurtleProperty::Size | TurtleProperty::Color);
    }

    // setup-patches
    let mut patches: Vec<_> = world.patches.patch_ids_iter().collect();
    for patch in ShuffleIterator::new(&mut patches, Rc::clone(&workspace.rng)) {
        // setup-nest

        // setup-food

        // recolor-patch
    }

    // TODO add the rest of the model

    drop(world);
    drop(workspace);
    w
}

fn main() {
    run_ants_model();
}
