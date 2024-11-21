use std::cell::RefCell;
use std::rc::Rc;

use oxitortoise::agent_variables::VariableDescriptor;
use oxitortoise::color::Color;
use oxitortoise::shuffle_iterator::ShuffleIterator;
use oxitortoise::topology::{Point, Topology};
use oxitortoise::turtle::{Shape, BREED_NAME_TURTLES};
use oxitortoise::updater::{PatchProperty, PrintUpdate, TurtleProperty, Update};
use oxitortoise::value;
use oxitortoise::workspace::Workspace;
use oxitortoise::{color, topology};

// define the Ants model. this is a direct translation of this code
// https://github.com/NetLogo/Tortoise/blob/master/resources/test/dumps/Ants.js
#[allow(unused_variables)]
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
        turtle.set_color(Color::RED);

        updater.update_turtle(&*turtle, TurtleProperty::Size | TurtleProperty::Color);
    }

    // setup-patches
    let mut patches: Vec<_> = world.patches.patch_ids_iter().collect();
    for &mut patch in ShuffleIterator::new(&mut patches, Rc::clone(&workspace.rng)) {
        // set nest? (distancexy 0 0) < 5
        let local_0 = topology::euclidean_distance_unwrapped(
            world.patches[patch].position().into(),
            Point { x: 0.0, y: 0.0 },
        );
        let local_1 = local_0 < 5.0;
        world.patches[patch].set_custom(patch_nest, value::Boolean(local_1).into());

        // set nest-scent 200 - distancexy 0 0
        let local_2 = 200.0 - local_0;
        world.patches[patch].set_custom(patch_nest_scent, value::Float::new(local_2).into());

        // setup-food

        let max_pxcor = world.topology.max_pxcor() as f64;
        let max_pycor = world.topology.max_pycor as f64;

        // ;; setup food source one on the right
        // if (distancexy (0.6 * max-pxcor) 0) < 5
        // [ set food-source-number 1 ]
        if topology::euclidean_distance_unwrapped(
            world.patches[patch].position().into(),
            Point {
                x: 0.6 * max_pxcor,
                y: 0.0,
            },
        ) < 5.0
        {
            world.patches[patch]
                .set_custom(patch_food_source_number, value::Float::new(1.0).into());
        }

        // ;; setup food source two on the lower-left
        // if (distancexy (-0.6 * max-pxcor) (-0.6 * max-pycor)) < 5
        // [ set food-source-number 2 ]
        if topology::euclidean_distance_unwrapped(
            world.patches[patch].position().into(),
            Point {
                x: -0.6 * max_pxcor,
                y: -0.6 * max_pycor,
            },
        ) < 5.0
        {
            world.patches[patch]
                .set_custom(patch_food_source_number, value::Float::new(2.0).into());
        }

        // ;; setup food source three on the upper-left
        // if (distancexy (-0.8 * max-pxcor) (0.8 * max-pycor)) < 5
        // [ set food-source-number 3 ]
        if topology::euclidean_distance_unwrapped(
            world.patches[patch].position().into(),
            Point {
                x: -0.8 * max_pxcor,
                y: 0.8 * max_pycor,
            },
        ) < 5.0
        {
            world.patches[patch]
                .set_custom(patch_food_source_number, value::Float::new(3.0).into());
        }

        // ;; set "food" at sources to either 1 or 2, randomly
        // if food-source-number > 0
        // [ set food one-of [1 2] ]
        if *world.patches[patch]
            .get_custom(patch_food_source_number)
            .get::<value::Float>()
            .unwrap()
            > value::Float::new(0.0)
        {
            let rand_index = workspace.rng.borrow_mut().next_int(2);
            let food_value = match rand_index {
                0 => value::Float::new(1.0),
                1 => value::Float::new(2.0),
                _ => unreachable!("rand_index should be 0 or 1"),
            };
            world.patches[patch].set_custom(patch_food, food_value.into());
        }

        // recolor-patch

        // ;; give color to nest and food sources
        // ifelse nest?
        // [ set pcolor violet ]
        // [ ifelse food > 0
        //     [ if food-source-number = 1 [ set pcolor cyan ]
        //     if food-source-number = 2 [ set pcolor sky  ]
        //     if food-source-number = 3 [ set pcolor blue ] ]
        //     ;; scale color to show chemical concentration
        //     [ set pcolor scale-color green chemical 0.1 5 ] ]
        if world.patches[patch]
            .get_custom(patch_nest)
            .get::<value::Boolean>()
            .unwrap()
            .0
        {
            world.patches[patch].set_pcolor(Color::VIOLET);
        } else if *world.patches[patch]
            .get_custom(patch_food)
            .get::<value::Float>()
            .unwrap()
            > value::Float::new(0.0)
        {
            let food_source_number = *world.patches[patch]
                .get_custom(patch_food_source_number)
                .get::<value::Float>()
                .unwrap();
            if food_source_number == value::Float::new(1.0) {
                world.patches[patch].set_pcolor(Color::CYAN);
            }
            if food_source_number == value::Float::new(2.0) {
                world.patches[patch].set_pcolor(Color::SKY);
            }
            if food_source_number == value::Float::new(3.0) {
                world.patches[patch].set_pcolor(Color::BLUE);
            }
        } else {
            let chemical = *world.patches[patch]
                .get_custom(patch_chemical)
                .get::<value::Float>()
                .unwrap();
            let scaled_color = color::scale_color(
                Color::GREEN,
                chemical,
                value::Float::new(0.1),
                value::Float::new(5.0),
            );
            world.patches[patch].set_pcolor(scaled_color);
        }

        updater.update_patch(&world.patches[patch], PatchProperty::Pcolor.into());
    }

    // TODO add the rest of the model

    drop(world);
    drop(workspace);
    w
}

fn main() {
    run_ants_model();
}
