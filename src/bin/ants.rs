use std::cell::RefCell;
use std::rc::Rc;

use oxitortoise::agent_variables::VariableDescriptor;
use oxitortoise::shuffle_iterator::ShuffleIterator;
use oxitortoise::topology::{Point, Topology};
use oxitortoise::turtle::{Shape, BREED_NAME_TURTLES};
use oxitortoise::updater::{PrintUpdate, TurtleProperty, Update};
use oxitortoise::value;
use oxitortoise::workspace::Workspace;
use oxitortoise::{color, topology};

// define the Ants model. this is a direct translation of this code
// https://github.com/NetLogo/Tortoise/blob/master/resources/test/dumps/Ants.js
fn run_ants_model() -> Rc<RefCell<Workspace>> {
    let w = Workspace::new(Topology {
        min_pxcor: -12,
        max_pycor: 12,
        world_width: 25,
        world_height: 25,
    });
    let workspace = w.borrow_mut();
    let mut world = workspace.world.borrow_mut();
    let mut updater = PrintUpdate;

    // declare widget variable
    world
        .observer
        .create_widget_global(Rc::from("population"), value::Float::new(2.0).into());

    // `patches-own [...]`
    let patch_var_names = [
        Rc::from("chemical"),
        Rc::from("food"),
        Rc::from("nest?"),
        Rc::from("nest-scent"),
        Rc::from("food-source-number"),
    ];
    world
        .patches
        .declare_custom_variables(patch_var_names.to_vec());
    let &[patch_chemical, patch_food, patch_nest, patch_nest_scent, patch_food_source_number] =
        patch_var_names
            .into_iter()
            .map(|name| {
                let Some(VariableDescriptor::Custom(var_idx)) =
                    world.patches.look_up_variable(&name)
                else {
                    unreachable!("variable should exist");
                };
                var_idx
            })
            .collect::<Vec<_>>()
            .as_slice()
    else {
        unreachable!("the length of the array is correct");
    };

    // `clear-all`
    world.clear_all();

    // `set-default-shape turtles "bug"`
    world
        .turtles
        .get_breed(BREED_NAME_TURTLES)
        .expect("default turtle breed should exist")
        .borrow_mut()
        .set_default_shape(Rc::new(Shape {}));

    // `create-turtles ...`
    let num = world
        .observer
        .get_global("population")
        .expect("compiler assumes this variable exists")
        .get::<value::Float>()
        .expect("compiler assumes this is a number")
        .to_u64_round_to_zero();
    let mut new_turtles = Vec::new();
    world.turtles.create_turtles(
        num,
        BREED_NAME_TURTLES,
        Point::ORIGIN,
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
    for &mut patch in ShuffleIterator::new(&mut patches, Rc::clone(&workspace.rng)) {
        let patch = &mut world.patches[patch];

        // setup-nest
        let local_0 = topology::euclidean_distance_unwrapped(
            patch.position().into(),
            Point { x: 0.0, y: 0.0 },
        );
        let local_1 = local_0 < 5.0;
        patch.set_custom(patch_nest, value::Boolean(local_1).into());

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
