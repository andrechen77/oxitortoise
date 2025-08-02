use std::rc::Rc;

use flagset::FlagSet;
use slotmap::SlotMap;
use walrus::Module;

use oxitortoise::{
    exec::{
        dynamic_link,
        jit::{FunctionInstaller, JitCallback, JitEntry},
        CanonExecutionContext,
    },
    sim::{
        agent_schema::{PatchSchema, TurtleSchema},
        patch::Patches,
        shapes::Shapes,
        tick::Tick,
        topology::{Point, Topology, TopologySpec},
        turtle::{Breed, TurtleId, Turtles},
        value::{Float, NetlogoInternalType},
        world::World,
    },
    updater::{DirtyAggregator, TurtleProp},
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

static mut CONTEXT: Option<CanonExecutionContext> = None;
static mut SETUP_FN: Option<JitEntry> = None;
static mut GO_FN: Option<JitEntry> = None;

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
    let entrypoints = &[
        module.funcs.by_name("shim_setup").unwrap(),
        module.funcs.by_name("shim_go").unwrap(),
    ];
    let potential_callbacks = &[
        module.funcs.by_name("setup_body0").unwrap(),
        module.funcs.by_name("setup_body1").unwrap(),
        module.funcs.by_name("go_body0").unwrap(),
        module.funcs.by_name("go_body1").unwrap(),
    ];
    let table_to_install = module.tables.main_function_table().unwrap().unwrap();
    let mut new_functions = unsafe {
        function_installer
            .install_functions(module, entrypoints, potential_callbacks, table_to_install)
            .unwrap()
    };
    real_print("installed new functions");
    // SAFETY: no one else has a reference to this variable since this is called
    // in single-threaded contexts only and this function is the entry point.
    unsafe {
        GO_FN = Some(new_functions.remove(1));
        SETUP_FN = Some(new_functions.remove(0));
    }

    // create the workspace and execution context
    let workspace = Box::leak(Box::new(create_workspace()));
    let mut dirty_aggregator = DirtyAggregator::new();
    dirty_aggregator.world |= FlagSet::full();
    dirty_aggregator.tick = workspace.world.tick_counter.clone();
    dirty_aggregator.reserve_turtles(1000);
    dirty_aggregator.reserve_patches(10000);
    // real_print(format!("Updater: {:?}", &updater));
    let rng = workspace.rng.clone();
    let context = CanonExecutionContext {
        workspace,
        dirty_aggregator,
        next_int: rng,
    };
    // SAFETY: no one else has a reference to this variable since this is called
    // in single-threaded contexts only and this function is the entry point.
    unsafe {
        CONTEXT = Some(context);
    }
}

#[no_mangle]
#[allow(static_mut_refs)]
pub fn run_setup_and_go() {
    // SAFETY: no one else has a reference to this variable since this is called
    // in single-threaded contexts only and this function is the entry point.
    let context = unsafe { CONTEXT.as_mut().unwrap() };
    let setup_fn = unsafe { SETUP_FN.as_ref().unwrap() };
    let go_fn = unsafe { GO_FN.as_ref().unwrap() };
    // call the dynamically loaded functions
    setup_fn.call(context, std::ptr::null_mut());
    // real_print(format!("After setup: {:?}", &context.updater));
    for _ in 0..1000 {
        go_fn.call(context, std::ptr::null_mut());
    }
    // real_print(format!("After go: {:?}", &context.updater));
}

#[no_mangle]
#[allow(static_mut_refs)]
pub fn run_diy() {
    let context = unsafe { CONTEXT.as_mut().unwrap() };
    dynamic_link::oxitortoise_clear_all(context);
    extern "C" fn body0(_env: *mut u8, context: &mut CanonExecutionContext, turtle_id: u64) {
        let turtle_id = TurtleId::from_ffi(turtle_id);
        let turtle_data = context
            .workspace
            .world
            .turtles
            .get_turtle_base_data_mut(turtle_id)
            .unwrap();
        turtle_data.color = Float::new(15.0).into();
        turtle_data.size = Float::new(2.0);
        context.dirty_aggregator.get_turtles_mut()[turtle_id.index()] |= TurtleProp::Color;
    }
    let breed = dynamic_link::oxitortoise_get_default_turtle_breed(context);
    dynamic_link::oxitortoise_create_turtles(
        context,
        breed,
        125,
        Point { x: 0.0, y: 0.0 },
        JitCallback {
            fn_ptr: body0,
            env: std::ptr::null_mut(),
        },
    )
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
