use std::cell::RefCell;
use std::rc::Rc;

use flagset::FlagSet;
use oxitortoise::agent::Agent;
use oxitortoise::agent_id::AgentId;
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
        world_width: 25,
        world_height: 25,
    });
    let mut workspace = w.borrow_mut();

    // `globals [ population ]`
    workspace
        .world
        .borrow_mut()
        .observer
        .create_widget_global(Rc::from("population"), Value::Float(value::Float(2.0)));

    // to setup
    let setup = |workspace: &mut Workspace, update: &mut dyn Update| {
        let mut world = workspace.world.borrow_mut();

        // `clear-all`
        world.clear_all();

        // `set-default-shape turtles "bug"`
        world
            .turtle_manager
            .set_default_shape(BREED_NAME_TURTLES, Shape {});

        // `create-turtles ...`
        let num = world
            .observer
            .get_global("population")
            .expect("compiler assumes this variable exists")
            .to_u64_round_to_zero()
            .expect("compiler assumes this is a number");
        let mut new_turtles = Vec::new();
        world.turtle_manager.create_turtles(
            num,
            BREED_NAME_TURTLES,
            0.0,
            0.0,
            |turtle| {
                new_turtles.push(turtle.clone());
                update.update_turtle(&turtle.borrow(), FlagSet::default());
            },
            &mut *workspace.rng.borrow_mut(),
        );

        // });
        for turtle in new_turtles {
            let mut turtle = turtle.borrow_mut();

            // `set size 2`
            turtle.set_size(2.0);

            // `set color red`
            turtle.set_color(color::RED);

            update.update_turtle(&*turtle, TurtleProperty::Size | TurtleProperty::Color);

            // on_creation(world, turtle.into(), update);
        }
    };
    setup(&mut *workspace, &mut PrintUpdate);

    // TODO add the rest of the model

    drop(workspace);
    w
}

fn main() {
    run_ants_model();
}
