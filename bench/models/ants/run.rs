use std::rc::Rc;

use engine::{
    mir::lowering::lower,
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
    let ast = include_str!("ast.json");
    let ast = ast_to_mir::serde_json::from_str(ast).unwrap();
    let mir = ast_to_mir::ast_to_mir(&ast).unwrap();

    for (fn_id, function) in &mir.functions {
        lower(&mut *function.borrow_mut(), &mir);
    }

    for (fn_id, function) in &mir.functions {
        let function = function.borrow();
        let dot_string = function.to_dot_string_with_options(false);
        let filename =
            format!("{}-{:?}.dot", fn_id, function.debug_name.as_deref().unwrap_or("unnamed"));
        std::fs::write(filename, dot_string).expect("Failed to write DOT file");
    }
}
