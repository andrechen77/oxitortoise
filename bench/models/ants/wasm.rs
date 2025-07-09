use std::rc::Rc;
use std::sync::OnceLock;

use flagset::FlagSet;
use oxitortoise::exec::CanonExecutionContext;
use oxitortoise::sim::agent_schema::{AgentFieldDescriptor, PatchSchema, TurtleSchema};
use oxitortoise::sim::patch::{PatchId, Patches};
use oxitortoise::sim::shapes::Shapes;
use oxitortoise::sim::tick::Tick;
use oxitortoise::sim::topology::{Heading, Point, Topology};
use oxitortoise::sim::turtle::{Breed, BreedId, Turtles};
use oxitortoise::sim::value::agentset::{AllPatches, AllTurtles, IterateAgentset as _};
use oxitortoise::sim::value::{Boolean, Float, NetlogoInternalType};
use oxitortoise::sim::world::World;
use oxitortoise::util::cell::RefCell;
use oxitortoise::util::rng::{CanonRng, Rng as _};
use oxitortoise::{
    exec::scripting::prelude as s,
    sim::{
        color::{self, Color},
        topology::TopologySpec,
    },
    updater::{PatchProp, TurtleProp, UpdateAggregator, WriteUpdate},
    workspace::Workspace,
};
use slotmap::SlotMap;

const PATCH_CHEMICAL: AgentFieldDescriptor = AgentFieldDescriptor {
    buffer_idx: 2,
    field_idx: 0,
};
const PATCH_FOOD: AgentFieldDescriptor = AgentFieldDescriptor {
    buffer_idx: 0,
    field_idx: 1,
};
const PATCH_NEST: AgentFieldDescriptor = AgentFieldDescriptor {
    buffer_idx: 0,
    field_idx: 2,
};
const PATCH_NEST_SCENT: AgentFieldDescriptor = AgentFieldDescriptor {
    buffer_idx: 0,
    field_idx: 3,
};
const PATCH_FOOD_SOURCE_NUMBER: AgentFieldDescriptor = AgentFieldDescriptor {
    buffer_idx: 0,
    field_idx: 4,
};
static DEFAULT_TURTLE_BREED: OnceLock<BreedId> = OnceLock::new();

#[no_mangle]
#[inline(never)]
fn create_workspace() -> Workspace {
    let topology_spec = TopologySpec {
        min_pxcor: -35,
        max_pycor: 35,
        patches_width: 71,
        patches_height: 71,
        wrap_x: false,
        wrap_y: false,
    };
    let patch_schema = PatchSchema::new(
        1,
        &[
            (NetlogoInternalType::FLOAT, 2),   // chemical
            (NetlogoInternalType::FLOAT, 0),   // food
            (NetlogoInternalType::BOOLEAN, 0), // nest?
            (NetlogoInternalType::FLOAT, 0),   // nest-scent
            (NetlogoInternalType::FLOAT, 0),   // food-source-number
        ],
        &[1, 2],
    );
    let turtle_schema = TurtleSchema::default();
    let patches = Patches::new(patch_schema, &topology_spec);
    let turtle_breeds = {
        let mut breeds = SlotMap::with_key();
        let key = breeds.insert(Breed {
            name: Rc::from("turtles"),
            singular_name: Rc::from("turtle"),
            active_custom_fields: vec![],
        });
        DEFAULT_TURTLE_BREED.set(key).unwrap();
        breeds
    };
    let turtles = Turtles::new(turtle_schema, turtle_breeds);
    let topology = Topology::new(topology_spec);
    let tick_counter = Tick::default();
    let shapes = Shapes::default();
    let world = World {
        turtles,
        patches,
        topology,
        tick_counter,
        shapes,
    };
    let rng = Rc::new(RefCell::new(CanonRng::new(0)));
    let workspace = Workspace { world, rng };

    // TODO declare the population widget variable

    workspace
}

#[no_mangle]
#[inline(never)]
fn setup(context: &mut CanonExecutionContext) {
    // clear-all
    s::clear_all(context);

    // create-turtles
    let new_turtles = s::create_turtles(
        context,
        *DEFAULT_TURTLE_BREED.get().unwrap(),
        125,
        Point::ORIGIN,
    );
    for turtle_id in new_turtles.into_iter(&context.workspace.world, context.next_int.clone()) {
        let base_data = context
            .workspace
            .world
            .turtles
            .get_turtle_base_data_mut(turtle_id)
            .unwrap();
        base_data.size = Float::new(2.0);
        base_data.color = Color::RED;
        context.updater.update_turtle(
            &context.workspace.world,
            turtle_id,
            TurtleProp::Size | TurtleProp::Color,
        );
    }

    // setup-patches
    for patch_id in AllPatches.into_iter(&context.workspace.world, context.next_int.clone()) {
        // calculate distancexy 0 0
        let this_patch = context
            .workspace
            .world
            .patches
            .get_patch_base_data(patch_id)
            .unwrap();
        let pxcor = this_patch.position.x;
        let pycor = this_patch.position.y;
        let this_patch_point = Point {
            x: pxcor as f64,
            y: pycor as f64,
        };
        let distance_origin = s::distance_euclidean_no_wrap(this_patch_point, Point::ORIGIN);

        // set nest? (distancexy 0 0) < 5
        {
            let condition = Boolean(distance_origin < Float::new(5.0));
            context
                .workspace
                .world
                .patches
                .set_patch_field(patch_id, PATCH_NEST, condition);
        }

        // set nest-scent 200 - distancexy 0 0
        {
            let nest_scent = Float::new(200.0) - distance_origin;
            let field: &mut Float = context
                .workspace
                .world
                .patches
                .get_patch_field_mut(patch_id, PATCH_NEST_SCENT)
                .unwrap()
                .unwrap_left();
            *field = nest_scent;
        }

        // setup-food
        {
            let max_pxcor = s::max_pxcor(&context.workspace.world);
            let max_pycor = s::max_pycor(&context.workspace.world);

            // if (distancexy (0.6 * max-pxcor) 0) < 5 [ set food-source-number 1 ]
            {
                let x = 0.6 * max_pxcor.get();
                let y = 0.0;
                let distance = s::distance_euclidean_no_wrap(this_patch_point, Point { x, y });
                if distance < Float::new(5.0) {
                    let field: &mut Float = context
                        .workspace
                        .world
                        .patches
                        .get_patch_field_mut(patch_id, PATCH_FOOD_SOURCE_NUMBER)
                        .unwrap()
                        .unwrap_left();
                    *field = Float::new(1.0);
                }
            }

            // if (distancexy (-0.6 * max-pxcor) (-0.6 * max-pycor)) < 5 [ set food-source-number 2 ]
            {
                let x = -0.6 * max_pxcor.get();
                let y = -0.6 * max_pycor.get();
                let distance = s::distance_euclidean_no_wrap(this_patch_point, Point { x, y });
                if distance < Float::new(5.0) {
                    let field: &mut Float = context
                        .workspace
                        .world
                        .patches
                        .get_patch_field_mut(patch_id, PATCH_FOOD_SOURCE_NUMBER)
                        .unwrap()
                        .unwrap_left();
                    *field = Float::new(2.0);
                }
            }

            // if (distancexy (-0.8 * max-pxcor) (0.8 * max-pycor)) < 5 [ set food-source-number 3 ]
            {
                let x = -0.8 * max_pxcor.get();
                let y = 0.8 * max_pycor.get();
                let distance = s::distance_euclidean_no_wrap(this_patch_point, Point { x, y });
                if distance < Float::new(5.0) {
                    let field: &mut Float = context
                        .workspace
                        .world
                        .patches
                        .get_patch_field_mut(patch_id, PATCH_FOOD_SOURCE_NUMBER)
                        .unwrap()
                        .unwrap_left();
                    *field = Float::new(3.0);
                }
            }

            // if food-source-number > 0 [ set food one-of [1 2] ]
            {
                let food_source_number: Float = *context
                    .workspace
                    .world
                    .patches
                    .get_patch_field(patch_id, PATCH_FOOD_SOURCE_NUMBER)
                    .unwrap()
                    .unwrap_left();
                if food_source_number > Float::new(0.0) {
                    let rand_index = context.next_int.borrow_mut().next_int(2);
                    let food_value = match rand_index {
                        0 => Float::new(1.0),
                        1 => Float::new(2.0),
                        _ => unreachable!("rand_index should be 0 or 1"),
                    };
                    let field: &mut Float = context
                        .workspace
                        .world
                        .patches
                        .get_patch_field_mut(patch_id, PATCH_FOOD)
                        .unwrap()
                        .unwrap_left();
                    *field = food_value;
                }
            }

            // recolor-patch
            recolor_patch(context, patch_id);
        }
    }

    // reset-ticks
    s::reset_ticks(&context.workspace.world);
    context
        .updater
        .update_tick(context.workspace.world.tick_counter.clone());
}

#[no_mangle]
#[inline(never)]
fn recolor_patch(context: &mut CanonExecutionContext, executor: PatchId) {
    // ifelse nest?
    let condition: Boolean = *context
        .workspace
        .world
        .patches
        .get_patch_field(executor, PATCH_NEST)
        .unwrap()
        .unwrap_left();
    if condition.0 {
        // set pcolor violet
        let field: &mut Color = context
            .workspace
            .world
            .patches
            .get_patch_pcolor_mut(executor)
            .unwrap();
        *field = Color::VIOLET;
    } else {
        // ifelse food > 0
        let food: Float = *context
            .workspace
            .world
            .patches
            .get_patch_field(executor, PATCH_FOOD)
            .unwrap()
            .unwrap_left();
        if food > Float::new(0.0) {
            // if food-source-number = 1 [ set pcolor cyan ]
            //   if food-source-number = 2 [ set pcolor sky  ]
            //   if food-source-number = 3 [ set pcolor blue ]
            let food_source_number: Float = *context
                .workspace
                .world
                .patches
                .get_patch_field(executor, PATCH_FOOD_SOURCE_NUMBER)
                .unwrap()
                .unwrap_left();
            let field: &mut Color = context
                .workspace
                .world
                .patches
                .get_patch_pcolor_mut(executor)
                .unwrap();
            if food_source_number == Float::new(1.0) {
                *field = Color::CYAN;
            }
            if food_source_number == Float::new(2.0) {
                *field = Color::SKY;
            }
            if food_source_number == Float::new(3.0) {
                *field = Color::BLUE;
            }
        } else {
            // set pcolor scale-color green chemical 0.1 5
            let chemical = *context
                .workspace
                .world
                .patches
                .get_patch_field(executor, PATCH_CHEMICAL)
                .unwrap()
                .unwrap_left();
            let scaled_color =
                color::scale_color(Color::GREEN, chemical, Float::new(0.1), Float::new(5.0));
            let field: &mut Color = context
                .workspace
                .world
                .patches
                .get_patch_pcolor_mut(executor)
                .unwrap();
            *field = scaled_color;
        }
    }

    context
        .updater
        .update_patch(&context.workspace.world, executor, PatchProp::Pcolor.into());
}

#[no_mangle]
#[inline(never)]
fn go(context: &mut CanonExecutionContext) {
    for turtle_id in AllTurtles.into_iter(&context.workspace.world, context.next_int.clone()) {
        // if who >= ticks [ stop ]
        let base_data = context
            .workspace
            .world
            .turtles
            .get_turtle_base_data(turtle_id)
            .unwrap();
        let who: Float = base_data.who.into();
        let Some(ticks) = context.workspace.world.tick_counter.get() else {
            panic!("ticks have not started yet");
        };
        if who >= ticks {
            return;
        }

        let mut position = *context
            .workspace
            .world
            .turtles
            .get_turtle_position(turtle_id)
            .unwrap();
        let mut heading = *context
            .workspace
            .world
            .turtles
            .get_turtle_heading(turtle_id)
            .unwrap();

        // ifelse color = red
        if base_data.color == Color::RED {
            // look-for-food
            {
                let patch_here_id = s::patch_at(&context.workspace.world, position.round_to_int());

                // if food > 0
                let food: Float = *context
                    .workspace
                    .world
                    .patches
                    .get_patch_field(patch_here_id, PATCH_FOOD)
                    .unwrap()
                    .unwrap_left();
                if food > Float::new(0.0) {
                    // set color orange + 1
                    let new_color = Color::ORANGE + Float::new(1.0);
                    let base_data = context
                        .workspace
                        .world
                        .turtles
                        .get_turtle_base_data_mut(turtle_id)
                        .unwrap();
                    base_data.color = new_color;

                    // set food food - 1
                    let new_food = food - Float::new(1.0);
                    let field: &mut Float = context
                        .workspace
                        .world
                        .patches
                        .get_patch_field_mut(patch_here_id, PATCH_FOOD)
                        .unwrap()
                        .unwrap_left();
                    *field = new_food;

                    // rt 180
                    heading += Float::new(180.0);
                    *context
                        .workspace
                        .world
                        .turtles
                        .get_turtle_heading_mut(turtle_id)
                        .unwrap() = heading;

                    // stop
                    return;
                }

                // if (chemical >= 0.05) and (chemical < 2)
                let chemical: Float = *context
                    .workspace
                    .world
                    .patches
                    .get_patch_field(patch_here_id, PATCH_CHEMICAL)
                    .unwrap()
                    .unwrap_left();
                if (chemical >= Float::new(0.05)) && (chemical < Float::new(2.0)) {
                    // uphill-chemical
                    uphill_patch_variable(context, position, &mut heading, PATCH_CHEMICAL);
                }
            }
        } else {
            // return-to-nest
            {
                let patch_here_id = s::patch_at(&context.workspace.world, position.round_to_int());

                // ifelse nest?
                let nest: Boolean = *context
                    .workspace
                    .world
                    .patches
                    .get_patch_field(patch_here_id, PATCH_NEST)
                    .unwrap()
                    .unwrap_left();
                if nest.0 {
                    // set color red
                    let base_data = context
                        .workspace
                        .world
                        .turtles
                        .get_turtle_base_data_mut(turtle_id)
                        .unwrap();
                    base_data.color = Color::RED;

                    // rt 180
                    heading += Float::new(180.0);
                    *context
                        .workspace
                        .world
                        .turtles
                        .get_turtle_heading_mut(turtle_id)
                        .unwrap() = heading;
                } else {
                    // set chemical chemical + 60
                    let field: &mut Float = context
                        .workspace
                        .world
                        .patches
                        .get_patch_field_mut(patch_here_id, PATCH_CHEMICAL)
                        .unwrap()
                        .unwrap_left();
                    *field += Float::new(60.0);

                    // uphill-nest-scent
                    uphill_patch_variable(context, position, &mut heading, PATCH_NEST_SCENT);
                }
            }
        }

        // wiggle
        {
            // rt random 40
            let rand_result = Float::from(s::random(context, 40) as i32);
            heading += rand_result;

            // lt random 40
            let rand_result = Float::from(s::random(context, 40) as i32);
            heading += -rand_result;

            // if not can-move? 1 [ rt 180 ]
            let point_ahead = s::offset_distance_by_heading(
                &context.workspace.world,
                position,
                heading,
                Float::new(1.0),
            );
            if point_ahead.is_none() {
                heading += Float::new(180.0);
            }

            // Update the turtle's heading
            *context
                .workspace
                .world
                .turtles
                .get_turtle_heading_mut(turtle_id)
                .unwrap() = heading;
        }

        // fd 1
        let new_position = s::offset_one_by_heading(&context.workspace.world, position, heading);
        if let Some(new_position) = new_position {
            position = new_position;
            *context
                .workspace
                .world
                .turtles
                .get_turtle_position_mut(turtle_id)
                .unwrap() = position;
        }

        context.updater.update_turtle(
            &context.workspace.world,
            turtle_id,
            TurtleProp::Position | TurtleProp::Heading | TurtleProp::Color,
        );
    }

    // diffuse chemical (diffusion-rate / 100)
    s::diffuse_8(
        &mut context.workspace.world,
        PATCH_CHEMICAL,
        Float::new(0.5),
    );

    // ask patches
    for patch_id in AllPatches.into_iter(&context.workspace.world, context.next_int.clone()) {
        // set chemical chemical * (100 - evaporation-rate) / 100
        let chemical: &mut Float = context
            .workspace
            .world
            .patches
            .get_patch_field_mut(patch_id, PATCH_CHEMICAL)
            .unwrap()
            .unwrap_left();
        *chemical *= 0.9;

        // recolor-patch
        recolor_patch(context, patch_id);
    }

    s::advance_tick(&context.workspace.world);
    context
        .updater
        .update_tick(context.workspace.world.tick_counter.clone());
}

#[no_mangle]
#[inline(never)]
fn patch_variable_at_angle(
    context: &CanonExecutionContext,
    position: Point,
    heading: Heading,
    angle: Float,
    patch_variable: AgentFieldDescriptor,
) -> Float {
    let real_heading = heading + angle;
    let point_ahead = s::offset_distance_by_heading(
        &context.workspace.world,
        position,
        real_heading,
        Float::new(1.0),
    );
    let patch_ahead = point_ahead.map(|point| {
        context
            .workspace
            .world
            .topology
            .patch_at(point.round_to_int())
    });
    match patch_ahead {
        None => Float::new(0.0),
        Some(patch_id) => {
            let patch_ahead = context
                .workspace
                .world
                .patches
                .get_patch_field(patch_id, patch_variable)
                .unwrap()
                .unwrap_left();
            *patch_ahead
        }
    }
}

#[no_mangle]
#[inline(never)]
fn uphill_patch_variable(
    context: &mut CanonExecutionContext,
    position: Point,
    heading: &mut Heading,
    patch_variable: AgentFieldDescriptor,
) {
    // let scent-ahead chemical-scent-at-angle 0
    // let scent-right nest-scent-at-angle  45
    // let scent-left  nest-scent-at-angle -45
    let scent_ahead =
        patch_variable_at_angle(context, position, *heading, Float::new(0.0), patch_variable);
    let scent_right = patch_variable_at_angle(
        context,
        position,
        *heading,
        Float::new(45.0),
        patch_variable,
    );
    let scent_left = patch_variable_at_angle(
        context,
        position,
        *heading,
        Float::new(-45.0),
        patch_variable,
    );

    // if (scent-right > scent-ahead) or (scent-left > scent-ahead)
    if (scent_right > scent_ahead) || (scent_left > scent_ahead) {
        // ifelse scent-right > scent-left
        if scent_right > scent_left {
            // rt 45
            *heading += Float::new(45.0);
        } else {
            // lt 45
            *heading += Float::new(-45.0);
        }
    }
}

// define the Ants model. this is a direct translation of this code
// https://github.com/NetLogo/Tortoise/blob/master/resources/test/dumps/Ants.js
#[allow(unused_variables)]
fn direct_run_ants() {
    let mut updater = UpdateAggregator::new();

    let mut workspace = create_workspace();

    updater.update_world_settings(&workspace.world, FlagSet::full());
    updater.update_tick(workspace.world.tick_counter.clone());

    let rng = workspace.rng.clone();
    let mut context = CanonExecutionContext {
        workspace: &mut workspace,
        updater,
        next_int: rng,
    };

    // run the `setup` function
    setup(&mut context);

    for _ in 0..1000 {
        go(&mut context);
    }
}

fn main() {
    direct_run_ants();
}
