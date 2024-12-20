use std::cell::RefCell;
use std::rc::Rc;

use flagset::FlagSet;
use oxitortoise::sim::agent::Agent;
use oxitortoise::updater::WorldProp;
use oxitortoise::util::rng::Rng as _;
use oxitortoise::{
    scripting::{self as s, ExecutionContext},
    sim::{
        agent_variables::VarIndex,
        color::{self, Color},
        topology::TopologySpec,
        turtle::BREED_NAME_TURTLES,
        value::{self, PolyValue},
    },
    updater::{PatchProp, TurtleProp, UpdateAggregator, WriteUpdate},
    workspace::Workspace,
};

const PATCH_CHEMICAL: VarIndex = VarIndex::from_index(0);
const PATCH_FOOD: VarIndex = VarIndex::from_index(1);
const PATCH_NEST: VarIndex = VarIndex::from_index(2);
const PATCH_NEST_SCENT: VarIndex = VarIndex::from_index(3);
const PATCH_FOOD_SOURCE_NUMBER: VarIndex = VarIndex::from_index(4);

fn create_workspace() -> Rc<RefCell<Workspace>> {
    let w = Workspace::new(TopologySpec {
        min_pxcor: -12,
        max_pycor: 12,
        patches_width: 25,
        patches_height: 25,
        wrap_x: false,
        wrap_y: false,
    });

    {
        let workspace = w.borrow_mut();
        let mut world = workspace.world.borrow_mut();

        // declare widget variable
        world
            .observer
            .borrow_mut()
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
    }

    w
}

fn setup<U: WriteUpdate>(context: &mut ExecutionContext<'_, U>) {
    // clear-all
    s::clear_all(context);

    // create-turtles
    s::create_turtles_with_cmd(
        context,
        value::Float::new(2.0),
        BREED_NAME_TURTLES,
        |context| {
            let Agent::Turtle(this_turtle) = context.executor else {
                panic!("must be executed by a turtle");
            };
            this_turtle.data.borrow_mut().size = value::Float::new(2.0);
            this_turtle.data.borrow_mut().color = Color::RED;
            context
                .updater
                .update_turtle(this_turtle.id(), FlagSet::full());
        },
    );

    // setup-patches
    s::ask(context, &mut value::agentset::AllPatches, |context| {
        let Agent::Patch(this_patch) = context.executor else {
            panic!("must be executed by a patch");
        };

        // set nest? (distancexy 0 0) < 5
        {
            let distance =
                s::distancexy_euclidean(this_patch, value::Float::new(0.0), value::Float::new(0.0));
            let condition: value::Boolean = (distance < value::Float::new(5.0)).into();
            let condition = PolyValue::from(condition);
            this_patch
                .data
                .borrow_mut()
                .set_custom(PATCH_NEST, condition);
        }

        // set nest-scent 200 - distancexy 0 0
        {
            let distance = s::distancexy_euclidean(this_patch, 0.0.into(), 0.0.into());
            let nest_scent = value::Float::new(200.0) - distance;
            this_patch
                .data
                .borrow_mut()
                .set_custom(PATCH_NEST_SCENT, nest_scent.into());
        }

        // setup-food
        {
            let max_pxcor = s::max_pxcor(context.world);
            let max_pycor = s::max_pycor(context.world);

            // if (distancexy (0.6 * max-pxcor) 0) < 5 [ set food-source-number 1 ]
            {
                let x = value::Float::new(0.6) * max_pxcor;
                let y = value::Float::new(0.0);
                let distance = s::distancexy_euclidean(this_patch, x, y);
                let condition = distance < value::Float::new(5.0);
                if condition {
                    this_patch.data.borrow_mut().set_custom(
                        PATCH_FOOD_SOURCE_NUMBER,
                        PolyValue::from(value::Float::new(1.0)),
                    );
                }
            }

            // if (distancexy (-0.6 * max-pxcor) (-0.6 * max-pycor)) < 5 [ set food-source-number 2 ]
            {
                let x = value::Float::new(-0.6) * max_pxcor;
                let y = value::Float::new(-0.6) * max_pycor;
                let distance = s::distancexy_euclidean(this_patch, x, y);
                let condition = distance < value::Float::new(5.0);
                if condition {
                    this_patch.data.borrow_mut().set_custom(
                        PATCH_FOOD_SOURCE_NUMBER,
                        PolyValue::from(value::Float::new(2.0)),
                    );
                }
            }

            // if (distancexy (-0.8 * max-pxcor) (0.8 * max-pycor)) < 5 [ set food-source-number 3 ]
            {
                let x = value::Float::new(-0.8) * max_pxcor;
                let y = value::Float::new(0.8) * max_pycor;
                let distance = s::distancexy_euclidean(this_patch, x, y);
                let condition = distance < value::Float::new(5.0);
                if condition {
                    this_patch.data.borrow_mut().set_custom(
                        PATCH_FOOD_SOURCE_NUMBER,
                        PolyValue::from(value::Float::new(3.0)),
                    );
                }
            }

            // TODO everything below here is just to make the model work and not
            // what the script would actually look like

            // if food-source-number > 0 [ set food one-of [1 2] ]
            {
                let food_source_number = *this_patch
                    .data
                    .borrow()
                    .get_custom(PATCH_FOOD_SOURCE_NUMBER)
                    .get::<value::Float>()
                    .unwrap();
                if food_source_number > value::Float::new(0.0) {
                    let rand_index = context.next_int.borrow_mut().next_int(2);
                    let food_value = match rand_index {
                        0 => value::Float::new(1.0),
                        1 => value::Float::new(2.0),
                        _ => unreachable!("rand_index should be 0 or 1"),
                    };
                    this_patch
                        .data
                        .borrow_mut()
                        .set_custom(PATCH_FOOD, PolyValue::from(food_value));
                }
            }
        }

        // recolor-patch
        {
            // ifelse nest?
            let condition = *this_patch
                .data
                .borrow()
                .get_custom(PATCH_NEST)
                .get::<value::Boolean>()
                .unwrap();
            if condition.0 {
                // set pcolor violet
                this_patch.data.borrow_mut().pcolor = Color::VIOLET;
            } else {
                // ifelse food > 0
                let food = *this_patch
                    .data
                    .borrow()
                    .get_custom(PATCH_FOOD)
                    .get::<value::Float>()
                    .unwrap();
                if food > value::Float::new(0.0) {
                    // if food-source-number = 1 [ set pcolor cyan ]
                    //   if food-source-number = 2 [ set pcolor sky  ]
                    //   if food-source-number = 3 [ set pcolor blue ]
                    let food_source_number = *this_patch
                        .data
                        .borrow()
                        .get_custom(PATCH_FOOD_SOURCE_NUMBER)
                        .get::<value::Float>()
                        .unwrap();
                    if food_source_number == value::Float::new(1.0) {
                        this_patch.data.borrow_mut().pcolor = Color::CYAN;
                    }
                    if food_source_number == value::Float::new(2.0) {
                        this_patch.data.borrow_mut().pcolor = Color::SKY;
                    }
                    if food_source_number == value::Float::new(3.0) {
                        this_patch.data.borrow_mut().pcolor = Color::BLUE;
                    }
                } else {
                    // set pcolor scale-color green chemical 0.1 5
                    let &chemical = this_patch
                        .data
                        .borrow()
                        .get_custom(PATCH_CHEMICAL)
                        .get::<value::Float>()
                        .unwrap();
                    let scaled_color = color::scale_color(
                        Color::GREEN,
                        chemical,
                        value::Float::new(0.1),
                        value::Float::new(5.0),
                    );
                    this_patch.data.borrow_mut().pcolor = scaled_color;
                }
            }
        }

        context
            .updater
            .update_patch(this_patch.id(), PatchProp::Pcolor.into());
    });

    // reset-ticks
    s::reset_ticks(context.world);
    context
        .updater
        .update_world_properties(WorldProp::Ticks.into());
}

fn go<U: WriteUpdate>(context: &mut ExecutionContext<'_, U>) {
    // TODO everything below here is just to make the model work and not
    // what the script would actually look like

    s::ask(context, &mut value::agentset::AllTurtles, |context| {
        let Agent::Turtle(this_turtle) = context.executor else {
            panic!("agent should be a turtle");
        };

        // if who >= ticks [ stop ]
        let who: value::Float = this_turtle.who().into();
        let Some(ticks) = context.world.tick_counter.get() else {
            panic!("ticks have not started yet");
        };
        if who >= ticks {
            return;
        }

        // ifelse color = red
        if this_turtle.data.borrow().color == Color::RED {
            // look-for-food
            look_for_food(context);
        } else {
            // return-to-nest
            return_to_nest(context);
        }

        // wiggle
        wiggle(context);

        // fd 1
        s::fd_one(context.world, this_turtle);

        context.updater.update_turtle(
            this_turtle.id(),
            TurtleProp::Position | TurtleProp::Heading | TurtleProp::Color,
        );
    });

    // diffuse chemical (diffusion-rate / 100)
    s::diffuse_8(context.world, PATCH_CHEMICAL, value::Float::new(0.5));

    // TODO finish script
    // ask patches
    // set chemical chemical * (100 - evaporation-rate) / 100
    // recolor-patch

    s::advance_tick(context.world);
    context
        .updater
        .update_world_properties(WorldProp::Ticks.into());
}

fn look_for_food<U: WriteUpdate>(context: &mut ExecutionContext<'_, U>) {
    let Agent::Turtle(this_turtle) = context.executor else {
        panic!("agent should be a turtle");
    };

    let patch_here_id = s::patch_here(context.world, this_turtle);
    let patch_here = s::look_up_patch(context.world, patch_here_id);

    // if food > 0
    let food = *patch_here
        .data
        .borrow()
        .get_custom(PATCH_FOOD)
        .get::<value::Float>()
        .unwrap();
    if food > value::Float::new(0.0) {
        // set color orange + 1
        let new_color = Color::ORANGE + value::Float::new(1.0);
        this_turtle.data.borrow_mut().color = new_color;

        // set food food - 1
        let new_food = food - value::Float::new(1.0);
        patch_here
            .data
            .borrow_mut()
            .set_custom(PATCH_FOOD, PolyValue::from(new_food));

        // rt 180
        s::turn(this_turtle, value::Float::new(180.0));

        // stop
        return;
    }

    // if (chemical >= 0.05) and (chemical < 2)
    let chemical = *patch_here
        .data
        .borrow()
        .get_custom(PATCH_CHEMICAL)
        .get::<value::Float>()
        .unwrap();
    if (chemical >= value::Float::new(0.05)) && (chemical < value::Float::new(2.0)) {
        // uphill-chemical
        uphill_patch_variable(context, PATCH_CHEMICAL);
    }
}

fn uphill_patch_variable<U: WriteUpdate>(
    context: &mut ExecutionContext<'_, U>,
    patch_variable: VarIndex,
) {
    let Agent::Turtle(this_turtle) = context.executor else {
        panic!("agent should be a turtle");
    };

    // let scent-ahead chemical-scent-at-angle 0
    // let scent-right nest-scent-at-angle  45
    // let scent-left  nest-scent-at-angle -45
    let scent_ahead = patch_variable_at_angle(context, value::Float::new(0.0), patch_variable);
    let scent_right = patch_variable_at_angle(context, value::Float::new(45.0), patch_variable);
    let scent_left = patch_variable_at_angle(context, value::Float::new(-45.0), patch_variable);

    // if (scent-right > scent-ahead) or (scent-left > scent-ahead)
    if (scent_right > scent_ahead) || (scent_left > scent_ahead) {
        // ifelse scent-right > scent-left
        if scent_right > scent_left {
            // rt 45
            s::turn(this_turtle, value::Float::new(45.0));
        } else {
            // lt 45
            s::turn(this_turtle, value::Float::new(-45.0));
        }
    }
}

fn patch_variable_at_angle<U: WriteUpdate>(
    context: &mut ExecutionContext<'_, U>,
    angle: value::Float,
    patch_variable: VarIndex,
) -> value::Float {
    let Agent::Turtle(this_turtle) = context.executor else {
        panic!("agent should be a turtle");
    };
    let patch_ahead = s::patch_at_angle(context.world, this_turtle, angle, value::Float::new(1.0));
    if let Some(patch_ahead) = patch_ahead {
        let patch_ahead = s::look_up_patch(context.world, patch_ahead);
        *patch_ahead
            .data
            .borrow()
            .get_custom(patch_variable)
            .get::<value::Float>()
            .unwrap()
    } else {
        value::Float::new(0.0)
    }
}

fn return_to_nest<U: WriteUpdate>(context: &mut ExecutionContext<'_, U>) {
    let Agent::Turtle(this_turtle) = context.executor else {
        panic!("agent should be a turtle");
    };

    // ifelse nest?
    let patch_id = s::patch_here(context.world, this_turtle);
    let patch = s::look_up_patch(context.world, patch_id);
    let nest = *patch
        .data
        .borrow()
        .get_custom(PATCH_NEST)
        .get::<value::Boolean>()
        .unwrap();
    if nest.0 {
        // set color red
        this_turtle.data.borrow_mut().color = Color::RED;

        // rt 180
        s::turn(this_turtle, value::Float::new(180.0));
    } else {
        // set chemical chemical + 60
        let chemical = *patch
            .data
            .borrow()
            .get_custom(PATCH_CHEMICAL)
            .get::<value::Float>()
            .unwrap();
        let new_chemical = chemical + value::Float::new(60.0);
        patch
            .data
            .borrow_mut()
            .set_custom(PATCH_CHEMICAL, PolyValue::from(new_chemical));

        // uphill-nest-scent
        uphill_patch_variable(context, PATCH_NEST_SCENT);
    }
}

fn wiggle<U: WriteUpdate>(context: &mut ExecutionContext<'_, U>) {
    let Agent::Turtle(this_turtle) = context.executor else {
        panic!("agent should be a turtle");
    };

    // rt random 40
    let rand_result = value::Float::from(s::random(context, 40));
    s::turn(this_turtle, rand_result);

    // lt random 40
    let rand_result = value::Float::from(s::random(context, 40));
    s::turn(this_turtle, -rand_result);

    // if not can-move? 1 [ rt 180 ]
    if !s::can_move(context.world, this_turtle, value::Float::new(1.0)).0 {
        s::turn(this_turtle, value::Float::new(180.0));
    }
}

// define the Ants model. this is a direct translation of this code
// https://github.com/NetLogo/Tortoise/blob/master/resources/test/dumps/Ants.js
#[allow(unused_variables)]
fn direct_run_ants() {
    let mut updater = UpdateAggregator::new();
    updater.update_world_properties(FlagSet::full());

    let w = create_workspace();

    let workspace = w.borrow_mut();
    let world = workspace.world.borrow_mut();
    let mut context = ExecutionContext {
        world: &world,
        executor: Agent::Observer(&world.observer),
        asker: Agent::Observer(&world.observer),
        updater,
        next_int: workspace.rng.clone(),
    };

    // run the `setup` function
    setup(&mut context);
    let update = context.updater.get_update(&world);
    println!("{:?}", update);

    for _ in 0..1000 {
        go(&mut context);

        let update = context.updater.get_update(&world);
        println!("{:?}", update);
    }
}

fn main() {
    direct_run_ants();
}
