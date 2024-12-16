use std::rc::Rc;
use std::{cell::RefCell, ops::DerefMut};

use oxitortoise::sim::agent::Agent;
use oxitortoise::util::rng::NextInt as _;
use oxitortoise::{
    scripting::{self as s, ExecutionContext},
    sim::{
        agent::{AgentId, AgentPosition},
        agent_variables::{VarIndex, VariableDescriptor},
        color::{self, Color},
        patch::{Patch, PatchId},
        topology::{self, Point, TopologySpec},
        turtle::{Shape, Turtle, TurtleId, BREED_NAME_TURTLES},
        value::{self, PolyValue},
        world::World,
    },
    updater::{PatchProperty, PrintUpdate, TurtleProperty, Update},
    util::shuffle_iterator::ShuffledMut,
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

fn setup<'w, U: Update>(context: &mut ExecutionContext<'w, U>) {
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
            let mut this_turtle = this_turtle.borrow_mut();
            this_turtle.size = value::Float::new(2.0);
            this_turtle.color = Color::RED;
            context.updater.update_turtle(
                this_turtle.deref_mut(),
                TurtleProperty::Size | TurtleProperty::Color,
            );
        },
    );

    // setup-patches
    s::ask(context, &mut value::agentset::AllPatches, |context| {
        let Agent::Patch(this_patch) = context.executor else {
            panic!("must be executed by a patch");
        };
        let mut this_patch = this_patch.borrow_mut();

        // set nest? (distancexy 0 0) < 5
        {
            let distance = s::distancexy_euclidean(
                &*this_patch,
                value::Float::new(0.0),
                value::Float::new(0.0),
            );
            let condition: value::Boolean = (distance < value::Float::new(5.0)).into();
            let condition = PolyValue::from(condition);
            Patch::set_custom(&mut *this_patch, PATCH_NEST, condition);
        }

        // set nest-scent 200
        {
            let lit = value::Float::new(200.0);
            let lit = PolyValue::from(lit);
            Patch::set_custom(&mut *this_patch, PATCH_NEST_SCENT, lit);
        }

        // setup-food
        {
            let max_pxcor: value::Float = value::Float::from(s::max_pxcor(context.world));
            let max_pycor = value::Float::from(s::max_pycor(context.world));

            // if (distancexy (0.6 * max-pxcor) 0) < 5 [ set food-source-number 1 ]
            {
                let x = value::Float::new(0.6) * max_pxcor;
                let y = value::Float::new(0.0);
                let distance = s::distancexy_euclidean(&mut *this_patch, x, y);
                let condition = distance < value::Float::new(5.0);
                if condition {
                    this_patch
                        .set_custom(PATCH_FOOD_SOURCE_NUMBER, PolyValue::from(value::Float::new(1.0)));
                }
            }

            // if (distancexy (-0.6 * max-pxcor) (-0.6 * max-pycor)) < 5 [ set food-source-number 2 ]
            {
                let x = value::Float::new(-0.6) * max_pxcor;
                let y = value::Float::new(-0.6) * max_pycor;
                let distance = s::distancexy_euclidean(&mut *this_patch, x, y);
                let condition = distance < value::Float::new(5.0);
                if condition {
                    this_patch
                        .set_custom(PATCH_FOOD_SOURCE_NUMBER, PolyValue::from(value::Float::new(2.0)));
                }
            }

            // if (distancexy (-0.8 * max-pxcor) (0.8 * max-pycor)) < 5 [ set food-source-number 3 ]
            {
                let x = value::Float::new(-0.8) * max_pxcor;
                let y = value::Float::new(0.8) * max_pycor;
                let distance = s::distancexy_euclidean(&mut *this_patch, x, y);
                let condition = distance < value::Float::new(5.0);
                if condition {
                    this_patch
                        .set_custom(PATCH_FOOD_SOURCE_NUMBER, PolyValue::from(value::Float::new(3.0)));
                }
            }

            // TODO everything below here is just to make the model work and not
            // what the script would actually look like

            // if food-source-number > 0 [ set food one-of [1 2] ]
            {
                let food_source_number = this_patch
                    .get_custom(PATCH_FOOD_SOURCE_NUMBER)
                    .get::<value::Float>()
                    .unwrap();
                if *food_source_number > value::Float::new(0.0) {
                    let rand_index = context.next_int.borrow_mut().next_int(2);
                    let food_value = match rand_index {
                        0 => value::Float::new(1.0),
                        1 => value::Float::new(2.0),
                        _ => unreachable!("rand_index should be 0 or 1"),
                    };
                    this_patch.set_custom(PATCH_FOOD, PolyValue::from(food_value));
                }
            }
        }

        // recolor-patch
        {
            // ifelse nest?
            let condition = this_patch
                .get_custom(PATCH_NEST)
                .get::<value::Boolean>()
                .unwrap();
            if condition.0 {
                // set pcolor violet
                this_patch.set_pcolor(Color::VIOLET);
            } else {
                // ifelse food > 0
                let food = this_patch
                    .get_custom(PATCH_FOOD)
                    .get::<value::Float>()
                    .unwrap();
                if *food > value::Float::new(0.0) {
                    // if food-source-number = 1 [ set pcolor cyan ]
                    //   if food-source-number = 2 [ set pcolor sky  ]
                    //   if food-source-number = 3 [ set pcolor blue ]
                    let &food_source_number = this_patch
                        .get_custom(PATCH_FOOD_SOURCE_NUMBER)
                        .get::<value::Float>()
                        .unwrap();
                    if food_source_number == value::Float::new(1.0) {
                        this_patch.set_pcolor(Color::CYAN);
                    }
                    if food_source_number == value::Float::new(2.0) {
                        this_patch.set_pcolor(Color::SKY);
                    }
                    if food_source_number == value::Float::new(3.0) {
                        this_patch.set_pcolor(Color::BLUE);
                    }
                } else {
                    // set pcolor scale-color green chemical 0.1 5
                    let &chemical = this_patch
                        .get_custom(PATCH_CHEMICAL)
                        .get::<value::Float>()
                        .unwrap();
                    let scaled_color = color::scale_color(
                        Color::GREEN,
                        chemical,
                        value::Float::new(0.1),
                        value::Float::new(5.0),
                    );
                    this_patch.set_pcolor(scaled_color);
                }
            }
        }

        context
            .updater
            .update_patch(&*this_patch, PatchProperty::Pcolor.into());
    });

    // reset-ticks
    s::reset_ticks(context.world);
}

fn go<'w, U: Update>(context: &mut ExecutionContext<'w, U>) {
    // TODO everything below here is just to make the model work and not
    // what the script would actually look like

    println!("tick is {}", context.world.tick_counter.get().unwrap());

    s::ask(context, &mut value::agentset::AllTurtles, |context| {
        let Agent::Turtle(this_turtle) = context.executor else {
            panic!("agent should be a turtle");
        };

        // if who >= ticks [ stop ]
        let who: value::Float = this_turtle.borrow().who().into();
        let Some(ticks) = context.world.tick_counter.get() else {
            panic!("ticks have not started yet");
        };
        if who >= ticks {
            return;
        }

        // ifelse color = red
        if this_turtle.borrow().color == Color::RED {
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

        context
            .updater
            .update_turtle(&*this_turtle.borrow(), TurtleProperty::Position.into());
    });

    s::advance_tick(context.world);
}

fn look_for_food<'w, U: Update>(context: &mut ExecutionContext<'w, U>) {
    let Agent::Turtle(this_turtle) = context.executor else {
        panic!("agent should be a turtle");
    };

    let patch_here_id = s::patch_here(&context.world, this_turtle);
    let patch_here = s::look_up_patch(&context.world, patch_here_id);

    // if food > 0
    let food = *patch_here.borrow().get_custom(PATCH_FOOD).get::<value::Float>().unwrap();
    if food > value::Float::new(0.0) {
        // set color orange + 1
        let new_color = Color::ORANGE + value::Float::new(1.0);
        this_turtle.borrow_mut().color = new_color;

        // set food food - 1
        let new_food = food - value::Float::new(1.0);
        patch_here.borrow_mut().set_custom(PATCH_FOOD, PolyValue::from(new_food));

        // rt 180
        s::turn(this_turtle, value::Float::new(180.0));

        // stop
        return;
    }

    // if (chemical >= 0.05) and (chemical < 2)
    let chemical = *patch_here.borrow().get_custom(PATCH_CHEMICAL).get::<value::Float>().unwrap();
    if (chemical >= value::Float::new(0.05)) && (chemical < value::Float::new(2.0)) {
        // uphill-chemical
        uphill_chemical(context);
    }
}

fn uphill_chemical<'w, U: Update>(context: &mut ExecutionContext<'w, U>) {
    println!("TODO: turtle is moving uphill chemical");
}

fn return_to_nest<'w, U: Update>(context: &mut ExecutionContext<'w, U>) {
    println!("TODO: turtle is returning to nest");
}

fn wiggle<'w, U: Update>(context: &mut ExecutionContext<'w, U>) {
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
    if !s::can_move(&context.world, this_turtle, value::Float::new(1.0)).0 {
        s::turn(this_turtle, value::Float::new(180.0));
    }
}

// define the Ants model. this is a direct translation of this code
// https://github.com/NetLogo/Tortoise/blob/master/resources/test/dumps/Ants.js
#[allow(unused_variables)]
fn direct_run_ants() {
    let updater = PrintUpdate;

    let w = create_workspace();

    let workspace = w.borrow_mut();
    let world = workspace.world.borrow_mut();
    let mut context = ExecutionContext {
        world: &*world,
        executor: Agent::Observer(&world.observer),
        asker: Agent::Observer(&world.observer),
        updater: updater,
        next_int: Rc::new(RefCell::new(oxitortoise::util::rng::CanonRng::new())),
    };

    // run the `setup` function
    setup(&mut context);

    // TODO repeatedly run the `go` function
    for _ in 0..10 {
        go(&mut context);
    }
}

fn main() {
    direct_run_ants();
}
