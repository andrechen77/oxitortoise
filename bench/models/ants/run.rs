use std::{cell::RefCell, rc::Rc};

use ast_to_mir::{ParseResult, add_cheats, serde_json, write_dot};
use engine::{
    mir::{
        mir_to_lir,
        transforms::{lower, optimize_of_agent_type, peephole_transform},
        type_inference::narrow_types,
    },
    sim::{
        patch::{PatchSchema, Patches},
        shapes::Shapes,
        tick::Tick,
        topology::{Topology, TopologySpec},
        turtle::{Breed, BreedId, TurtleSchema, Turtles},
        value::{NlBool, NlFloat},
        world::World,
    },
    slotmap::SlotMap,
    util::{reflection::Reflect as _, rng::CanonRng},
    workspace::Workspace,
};
use tracing::{Level, info};
use tracing_subscriber::{
    Layer as _, filter::Targets, layer::SubscriberExt as _, util::SubscriberInitExt as _,
};

// keep this function around as as reminder of how the workspace is set up
#[allow(dead_code)]
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
            (NlFloat::CONCRETE_TY, 2), // chemical
            (NlFloat::CONCRETE_TY, 0), // food
            (NlBool::CONCRETE_TY, 0),  // nest?
            (NlFloat::CONCRETE_TY, 0), // nest-scent
            (NlFloat::CONCRETE_TY, 0), // food-source-number
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

    for fn_id in program.functions.keys() {
        write_dot(&program, fn_id, "original");
    }

    info!("applying cheats");
    let cheats = include_str!("cheats.json");
    let cheats = serde_json::from_str(cheats).unwrap();
    add_cheats(&cheats, &mut program, &global_names, &fn_info);

    let fn_ids: Vec<_> = program.functions.keys().collect();
    for fn_id in fn_ids {
        info!(
            "transforming function {} {}",
            fn_id,
            program.functions[fn_id].debug_name.as_deref().unwrap_or_default()
        );
        narrow_types(&mut program);
        peephole_transform(&mut program, fn_id);
        optimize_of_agent_type(&mut program, fn_id);
        peephole_transform(&mut program, fn_id);
        lower(&mut program, fn_id);
    }

    for fn_id in program.functions.keys() {
        write_dot(&program, fn_id, "transformed");
    }

    std::fs::write("program.txt", format!("{:#?}", program)).unwrap();

    let lir_program = mir_to_lir(&program);
    println!("{}", lir_program);
}
