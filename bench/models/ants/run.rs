use std::rc::Rc;

use ast_to_mir::{ParseResult, add_cheats, serde_json, write_dot};
use engine::{
    mir::transforms::{lower, transform},
    sim::{
        agent_schema::{PatchSchema, TurtleSchema},
        patch::Patches,
        shapes::Shapes,
        tick::Tick,
        topology::{Topology, TopologySpec},
        turtle::{Breed, BreedId, Turtles},
        value::NetlogoMachineType,
        world::World,
    },
    slotmap::SlotMap,
    util::{cell::RefCell, rng::CanonRng},
    workspace::Workspace,
};
use tracing::{Level, info};
use tracing_subscriber::{
    Layer as _, filter::Targets, layer::SubscriberExt as _, util::SubscriberInitExt as _,
};

fn create_workspace() -> (Workspace, BreedId) {
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
            (NetlogoMachineType::FLOAT, 2),   // chemical
            (NetlogoMachineType::FLOAT, 0),   // food
            (NetlogoMachineType::BOOLEAN, 0), // nest?
            (NetlogoMachineType::FLOAT, 0),   // nest-scent
            (NetlogoMachineType::FLOAT, 0),   // food-source-number
        ],
        &[1, 2],
    );
    let turtle_schema = TurtleSchema::default();
    let patches = Patches::new(patch_schema, &topology_spec);
    let (turtle_breeds, default_turtle_breed) = {
        let mut breeds = SlotMap::with_key();
        let key = breeds.insert(Breed {
            name: Rc::from("turtles"),
            singular_name: Rc::from("turtle"),
            active_custom_fields: vec![],
        });
        (breeds, key)
    };
    let turtles = Turtles::new(turtle_schema, turtle_breeds);
    let topology = Topology::new(topology_spec);
    let tick_counter = Tick::default();
    let shapes = Shapes::default();
    let world = World { turtles, patches, topology, tick_counter, shapes };
    let rng = Rc::new(RefCell::new(CanonRng::new(0)));
    let workspace = Workspace { world, rng };

    // TODO declare the population widget variable

    (workspace, default_turtle_breed)
}

fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer().with_filter(
                Targets::new()
                    .with_target("oxitortoise_engine", Level::TRACE)
                    .with_target("oxitortoise_ast_to_mir", Level::INFO)
                    .with_target("ants", Level::TRACE)
                    .with_target("oxitortoise_main", Level::TRACE),
            ),
        )
        .init();

    info!("Hello world!");

    let ast = include_str!("ast.json");
    let ast = serde_json::from_str(ast).unwrap();
    let ParseResult { mut program, global_names, fn_info } = ast_to_mir::ast_to_mir(ast).unwrap();

    for (fn_id, function) in &program.functions {
        write_dot(fn_id, &*function.borrow());
    }

    info!("applying cheats");
    let cheats = include_str!("cheats.json");
    let cheats = serde_json::from_str(cheats).unwrap();
    add_cheats(&cheats, &mut program, &global_names, &fn_info);

    for (fn_id, function) in &program.functions {
        info!(
            "transforming function {} {}",
            fn_id,
            function.borrow().debug_name.as_deref().unwrap_or_default()
        );
        transform(&mut function.borrow_mut(), &program);
        lower(&mut function.borrow_mut(), &program);
    }

    for (fn_id, function) in &program.functions {
        write_dot(fn_id, &*function.borrow());
    }
}
