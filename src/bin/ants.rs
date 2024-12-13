use std::rc::Rc;
use std::{cell::RefCell, ops::DerefMut};

use oxitortoise::sim::agent::Agent;
use oxitortoise::util::rng::NextInt as _;
use oxitortoise::{
    scripting::{self, ExecutionContext},
    sim::{
        agent::{AgentId, AgentPosition},
        agent_variables::{VarIndex, VariableDescriptor},
        color::{self, Color},
        patch::{Patch, PatchId},
        topology::{self, Point, Topology},
        turtle::{Shape, Turtle, TurtleId, BREED_NAME_TURTLES},
        value::{self, PolyValue},
        world::World,
    },
    updater::{PatchProperty, PrintUpdate, TurtleProperty, Update},
    util::shuffle_iterator::ShuffledMut,
    workspace::Workspace,
};

fn create_workspace() -> Rc<RefCell<Workspace>> {
    let w = Workspace::new(Topology {
        min_pxcor: -12,
        max_pycor: 12,
        world_width: 25,
        world_height: 25,
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
    scripting::clear::clear_all(context);

    // create-turtles
    scripting::create_agent::create_turtles_with_cmd(
        context,
        value::Float::new(2.0),
        BREED_NAME_TURTLES,
        |context| {
            let Agent::Turtle(this_turtle) = context.executor else {
                panic!("must be executed by a turtle");
            };
            let mut this_turtle = this_turtle.borrow_mut();
            this_turtle.set_size(value::Float::new(2.0));
            this_turtle.set_color(Color::RED);
            context.updater.update_turtle(
                this_turtle.deref_mut(),
                TurtleProperty::Size | TurtleProperty::Color,
            );
        },
    );

    // setup-patches
    scripting::ask::ask(context, &mut value::agentset::AllPatches, |context| {
        let Agent::Patch(this_patch) = context.executor else {
            panic!("must be executed by a patch");
        };
        let mut this_patch = this_patch.borrow_mut();

        // set nest? (distancexy 0 0) < 5
        {
            let distance = scripting::topology::distancexy_euclidean(
                &*this_patch,
                value::Float::new(0.0),
                value::Float::new(0.0),
            );
            let condition: value::Boolean = (distance < value::Float::new(5.0)).into();
            let condition = PolyValue::from(condition);
            Patch::set_custom(&mut *this_patch, VarIndex::from_index(2), condition);
        }

        // set nest-scent 200
        {
            let lit = value::Float::new(200.0);
            let lit = PolyValue::from(lit);
            let patch_nest_scent = VarIndex::from_index(3);
            Patch::set_custom(&mut *this_patch, patch_nest_scent, lit);
        }

        // setup-food
        {
            let food_source_number = VarIndex::from_index(4);

            let max_pxcor: value::Float = value::Float::from(context.world.topology.max_pxcor());
            let max_pycor = value::Float::from(context.world.topology.max_pycor);

            // if (distancexy (0.6 * max-pxcor) 0) < 5 [ set food-source-number 1 ]
            {
                let x = value::Float::new(0.6) * max_pxcor;
                let y = value::Float::new(0.0);
                let distance = scripting::topology::distancexy_euclidean(&mut *this_patch, x, y);
                let condition = distance < value::Float::new(5.0);
                if condition {
                    this_patch
                        .set_custom(food_source_number, PolyValue::from(value::Float::new(1.0)));
                }
            }

            // if (distancexy (-0.6 * max-pxcor) (-0.6 * max-pycor)) < 5 [ set food-source-number 2 ]
            {
                let x = value::Float::new(-0.6) * max_pxcor;
                let y = value::Float::new(-0.6) * max_pycor;
                let distance = scripting::topology::distancexy_euclidean(&mut *this_patch, x, y);
                let condition = distance < value::Float::new(5.0);
                if condition {
                    this_patch
                        .set_custom(food_source_number, PolyValue::from(value::Float::new(2.0)));
                }
            }

            // if (distancexy (-0.8 * max-pxcor) (0.8 * max-pycor)) < 5 [ set food-source-number 3 ]
            {
                let x = value::Float::new(-0.8) * max_pxcor;
                let y = value::Float::new(0.8) * max_pycor;
                let distance = scripting::topology::distancexy_euclidean(&mut *this_patch, x, y);
                let condition = distance < value::Float::new(5.0);
                if condition {
                    this_patch
                        .set_custom(food_source_number, PolyValue::from(value::Float::new(3.0)));
                }
            }

            // TODO everything below here is just to make the model work and not
            // what the script would actually look like

            // if food-source-number > 0 [ set food one-of [1 2] ]
            {
                let food_source_number = this_patch
                    .get_custom(food_source_number)
                    .get::<value::Float>()
                    .unwrap();
                if *food_source_number > value::Float::new(0.0) {
                    let rand_index = context.next_int.borrow_mut().next_int(2);
                    let food_value = match rand_index {
                        0 => value::Float::new(1.0),
                        1 => value::Float::new(2.0),
                        _ => unreachable!("rand_index should be 0 or 1"),
                    };
                    this_patch.set_custom(VarIndex::from_index(1), PolyValue::from(food_value));
                }
            }
        }

        // recolor-patch
        {
            // ifelse nest?
            let condition = this_patch
                .get_custom(VarIndex::from_index(2))
                .get::<value::Boolean>()
                .unwrap();
            if condition.0 {
                // set pcolor violet
                this_patch.set_pcolor(Color::VIOLET);
            } else {
                // ifelse food > 0
                let food = this_patch
                    .get_custom(VarIndex::from_index(1))
                    .get::<value::Float>()
                    .unwrap();
                if *food > value::Float::new(0.0) {
                    // if food-source-number = 1 [ set pcolor cyan ]
                    //   if food-source-number = 2 [ set pcolor sky  ]
                    //   if food-source-number = 3 [ set pcolor blue ]
                    let &food_source_number = this_patch
                        .get_custom(VarIndex::from_index(4))
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
                        .get_custom(VarIndex::from_index(0))
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
    scripting::clear::reset_ticks(context);
}

fn go<'w, U: Update>(context: &mut ExecutionContext<'w, U>) {}

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
}

fn main() {
    direct_run_ants();
}
