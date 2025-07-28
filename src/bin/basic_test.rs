use std::rc::Rc;

use flagset::FlagSet;
use slotmap::SlotMap;
use walrus::Module;

use oxitortoise::{
    exec::{jit::FunctionInstaller, CanonExecutionContext},
    sim::{
        agent_schema::{PatchSchema, TurtleSchema},
        patch::Patches,
        shapes::Shapes,
        tick::Tick,
        topology::{Topology, TopologySpec},
        turtle::{Breed, Turtles},
        value::NetlogoInternalType,
        world::World,
    },
    updater::{UpdateAggregator, WriteUpdate as _},
    util::{cell::RefCell, rng::CanonRng},
    workspace::Workspace,
};

#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    fn write_to_console(message: *const u8, length: usize);

    safe fn get_model_code_size() -> usize;

    fn write_model_code(buffer: *mut u8);
}

#[cfg(target_arch = "wasm32")]
fn real_print(message: impl AsRef<str>) {
    let message = message.as_ref();
    let length = message.len();
    let message_ptr = message.as_ptr();
    unsafe {
        write_to_console(message_ptr, length);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn real_print(message: impl AsRef<str>) {
    println!("{}", message.as_ref());
}

#[cfg(not(target_arch = "wasm32"))]
fn get_model_code_size() -> usize {
    use std::fs;
    let wasm_bytes = fs::read("bench/models/ants/model_code.wasm").unwrap();
    wasm_bytes.len()
}

#[cfg(not(target_arch = "wasm32"))]
fn write_model_code(buffer: *mut u8) {
    use std::fs;
    let wasm_bytes = fs::read("bench/models/ants/model_code.wasm").unwrap();
    unsafe {
        std::ptr::copy_nonoverlapping(wasm_bytes.as_ptr(), buffer, wasm_bytes.len());
    }
}

pub fn main() {
    real_print("Hello, world!");

    // dynamically load the model code
    let module_bytes_len = get_model_code_size();
    let mut module_bytes = vec![0; module_bytes_len];
    unsafe {
        write_model_code(module_bytes.as_mut_ptr());
    }
    real_print("Loaded model code");
    let module = Module::from_buffer(&module_bytes).unwrap();
    let mut function_installer = unsafe { FunctionInstaller::new() };
    let functions_to_install = &[
        module.funcs.by_name("shim_setup").unwrap(),
        module.funcs.by_name("shim_go").unwrap(),
    ];
    let table_to_install = module.tables.main_function_table().unwrap().unwrap();
    let new_functions = unsafe {
        function_installer
            .install_functions(module, functions_to_install, table_to_install)
            .unwrap()
    };
    real_print("installed new functions");
    let setup_fn = &new_functions[0];
    let go_fn = &new_functions[1];

    // create the workspace and execution context
    let mut workspace = create_workspace();
    let mut updater = UpdateAggregator::new();
    updater.update_world_settings(&workspace.world, FlagSet::full());
    updater.update_tick(workspace.world.tick_counter.clone());
    real_print(format!("Updater: {:?}", &updater));
    let rng = workspace.rng.clone();
    let mut context = CanonExecutionContext {
        workspace: &mut workspace,
        updater,
        next_int: rng,
    };

    // call the dynamically loaded functions
    setup_fn.call(&mut context, std::ptr::null_mut());
    real_print(format!("Updater: {:?}", &context.updater));
}

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
