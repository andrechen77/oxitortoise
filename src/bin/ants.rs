use std::rc::Rc;

use flagset::FlagSet;
use oxitortoise::agent::Agent;
use oxitortoise::agent_id::AgentId;
use oxitortoise::procedure::{AnonCommand, Command};
use oxitortoise::turtle::{Shape, BREED_NAME_TURTLES};
use oxitortoise::updater::{PrintUpdate, TurtleProperty};
use oxitortoise::value::Value;
use oxitortoise::{color, value};
use oxitortoise::workspace::Workspace;

// define the Ants model. this is a direct translation of this code
// https://github.com/NetLogo/Tortoise/blob/master/resources/test/dumps/Ants.js
fn create_ants_model() -> Workspace {
    let mut workspace = Workspace::new();

    // `globals [ population ]`
    workspace.world.observer.create_global(Rc::from("population"), Value::Float(value::Float(2.0)));

    workspace.procedures.define_command(Command {
        name: Rc::from("setup"),
        start: 637,
        end: 715,
        action: Box::new(|world, _, update| {
            // `clear-all`
            world.clear_all();

            // `set-default-shape turtles "bug"`
            world
                .turtle_manager
                .set_default_shape(BREED_NAME_TURTLES, Shape {});

            // `create-turtles ...`
            let mut new_turtles = Vec::new();
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
                |turtle| {
                    new_turtles.push(turtle.id());
                    update.update_turtle(turtle, FlagSet::default())
                }
            );
            let on_creation: AnonCommand = Box::new(|world, executor, update| {
                // get the currently executing agent
                let executor = world.get_agent(executor).expect("a command should be executed by an existing agent");
                let Agent::Turtle(turtle) = executor else {
                    unreachable!("compiler assumes that only a turtle will run this command")
                };
                let mut turtle = turtle.borrow_mut();

                // `set size 2`
                turtle.set_size(2.0);

                // `set color red`
                turtle.set_color(color::RED);

                update.update_turtle(&*turtle, TurtleProperty::Size | TurtleProperty::Color);
            });
            for turtle in new_turtles {
                on_creation(world, turtle.into(), update);
            }
        }),
    });

    // TODO add the rest of the model

    workspace
}

fn main() {
    let mut workspace = create_ants_model();
    let setup = &workspace.procedures.get_command_by_name("setup").unwrap().action;
    setup(&mut workspace.world, AgentId::Observer, &mut PrintUpdate);
}
