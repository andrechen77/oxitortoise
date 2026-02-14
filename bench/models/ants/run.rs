use std::{
    io,
    sync::{Arc, Mutex},
};

use ast_to_mir::{NameReferent, ParseResult, add_cheats, serde_json};
use engine::{
    exec::{ExecutionContext, jit::InstallLir as _},
    mir::{
        TurtleBreeds, mir_to_lir,
        transforms::{lower, optimize_of_agent_type, peephole_transform},
        type_inference::narrow_types,
    },
    sim::{
        observer::{Globals, GlobalsSchema},
        patch::{PatchBaseData, PatchSchema, Patches},
        shapes::Shapes,
        tick::Tick,
        topology::{Topology, TopologySpec},
        turtle::{Breed, BreedId, TurtleBaseData, TurtleSchema, Turtles},
        value::NlFloat,
        world::World,
    },
    slotmap::SecondaryMap,
    updater::DirtyAggregator,
    util::{rng::CanonRng, row_buffer::RowBuffer},
    workspace::Workspace,
};
use oxitortoise_main::LirInstaller;
use tracing::{Level, error, info, trace};
use tracing_subscriber::{
    Layer as _, filter::Targets, fmt::MakeWriter, layer::SubscriberExt as _,
    util::SubscriberInitExt as _,
};

macro_rules! print_offsets {
    ($struct:ident, $($field:ident).*) => {
        let offset = std::mem::offset_of!($struct, $($field).*);
        tracing::trace!("offset of {}: {}", stringify!($struct.$($field).*), offset);
    }
}

fn print_offsets(workspace: &Workspace) {
    print_offsets!(Workspace, world);
    print_offsets!(World, globals.data);
    print_offsets!(World, turtles.data);
    print_offsets!(World, patches.data);
    print_offsets!(World, topology);
    print_offsets!(World, topology.max_x);
    print_offsets!(World, topology.max_y);
    print_offsets!(World, tick_counter);
    print_offsets!(World, shapes);
    for (i, row_buffer) in workspace
        .world
        .turtles
        .data
        .iter()
        .enumerate()
        .filter_map(|(i, r)| r.as_ref().map(|r| (i, r)))
    {
        trace!("turtle row buffer {}: {:?}", i, row_buffer.schema());
    }
    print_offsets!(TurtleBaseData, who);
    print_offsets!(TurtleBaseData, breed);
    print_offsets!(TurtleBaseData, shape_name);
    print_offsets!(TurtleBaseData, color);
    print_offsets!(TurtleBaseData, label);
    print_offsets!(TurtleBaseData, label_color);
    print_offsets!(TurtleBaseData, hidden);
    print_offsets!(TurtleBaseData, size);
    for (i, row_buffer) in workspace
        .world
        .patches
        .data
        .iter()
        .enumerate()
        .filter_map(|(i, r)| r.as_ref().map(|r| (i, r)))
    {
        trace!("patch row buffer {}: {:?}", i, row_buffer.schema());
    }
    print_offsets!(PatchBaseData, position);
    print_offsets!(PatchBaseData, plabel);
    print_offsets!(PatchBaseData, plabel_color);
    trace!("globals: {:?}", workspace.world.globals.data.schema());
    trace!("size of row buffer: {}", size_of::<RowBuffer>());
}

fn create_workspace(
    globals_schema: GlobalsSchema,
    turtle_schema: TurtleSchema,
    turtle_breeds: SecondaryMap<BreedId, Breed>,
    patch_schema: PatchSchema,
) -> Workspace {
    let topology_spec = TopologySpec {
        min_pxcor: -35,
        max_pycor: 35,
        patches_width: 71,
        patches_height: 71,
        wrap_x: false,
        wrap_y: false,
    };
    Workspace {
        world: World {
            globals: Globals::new(globals_schema),
            turtles: Turtles::new(turtle_schema, turtle_breeds),
            patches: Patches::new(patch_schema, &topology_spec),
            topology: Topology::new(topology_spec),
            tick_counter: Tick::default(),
            shapes: Shapes::default(),
        },
        rng: Arc::new(Mutex::new(CanonRng::new(0))),
    }
}

// static WORKSPACE: Mutex<Option<Workspace>> = Mutex::new(None);

fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(ConsoleWriterFactory)
                .without_time()
                .with_filter(
                    Targets::new()
                        .with_target("oxitortoise_engine", Level::INFO)
                        .with_target("oxitortoise_ast_to_mir", Level::INFO)
                        .with_target("ants", Level::TRACE)
                        .with_target("oxitortoise_main", Level::TRACE)
                        .with_target("oxitortoise_lir_to_wasm", Level::INFO),
                ),
        )
        .init();

    info!("Hello world!");

    let ast = include_str!("ast.json");
    let ast = serde_json::from_str(ast).unwrap();
    let ParseResult { mut program, global_names, fn_info } = ast_to_mir::ast_to_mir(ast).unwrap();

    info!("applying cheats");
    let cheats = include_str!("cheats.json");
    let cheats = serde_json::from_str(cheats).unwrap();
    add_cheats(&cheats, &mut program, &global_names, &fn_info);

    let mir_filename = "before.mir";
    let mir_str = program.pretty_print();
    write_to_file(mir_filename, mir_str);

    let fn_ids: Vec<_> = program.functions.keys().collect();
    narrow_types(&mut program);
    for fn_id in fn_ids {
        info!(
            "transforming function {} {}",
            fn_id,
            program.functions[fn_id].debug_name.as_deref().unwrap_or_default()
        );
        peephole_transform(&mut program, fn_id);
        optimize_of_agent_type(&mut program, fn_id);
        peephole_transform(&mut program, fn_id);
        lower(&mut program, fn_id);
    }
    let mir_filename = "after.mir";
    let mir_str = program.pretty_print();
    write_to_file(mir_filename, mir_str);

    let (lir_program, mir_to_lir_fns) = mir_to_lir::<LirInstaller>(&program);
    let lir_str = lir_program.pretty_print();
    let lir_filename = "model.lir";
    write_to_file(lir_filename, lir_str);

    // set up the workspace
    let TurtleBreeds::Full(breeds) = program.turtle_breeds else {
        panic!("turtle breeds are not full");
    };
    let mut workspace = create_workspace(
        program.globals_schema.unwrap(),
        program.turtle_schema.unwrap(),
        breeds,
        program.patch_schema.unwrap(),
    );

    print_offsets(&workspace);
    for (i, row_buffer) in workspace.world.patches.data.iter().enumerate() {
        trace!("patch row buffer {}: {:?}", i, row_buffer);
    }
    for (i, row_buffer) in workspace.world.turtles.data.iter().enumerate() {
        trace!("turtle row buffer {}: {:?}", i, row_buffer);
    }

    let mut lir_installer = LirInstaller::default();
    let result = unsafe { lir_installer.install_lir(&lir_program) };
    let name = "model.wasm";
    write_to_file(name, lir_installer.module_bytes);
    let functions = match result {
        Ok(functions) => {
            for fn_id in functions.keys() {
                info!("installed entrypoint function {:?}", fn_id);
            }
            functions
        }
        Err(_error) => {
            error!("failed to install LIR program");
            panic!();
        }
    };

    let NameReferent::Global(population) = global_names.lookup("POPULATION").unwrap() else {
        panic!("expected a global variable");
    };
    let NameReferent::Global(diffusion_rate) = global_names.lookup("DIFFUSION-RATE").unwrap()
    else {
        panic!("expected a global variable");
    };
    let NameReferent::Global(evaporation_rate) = global_names.lookup("EVAPORATION-RATE").unwrap()
    else {
        panic!("expected a global variable");
    };
    *workspace.world.globals.get_mut::<NlFloat>(population).unwrap_left() = NlFloat::new(125.0);
    *workspace.world.globals.get_mut::<NlFloat>(diffusion_rate).unwrap_left() = NlFloat::new(50.0);
    *workspace.world.globals.get_mut::<NlFloat>(evaporation_rate).unwrap_left() =
        NlFloat::new(10.0);
    visualize_update(workspace.world.generate_js_update_full());

    // *WORKSPACE.lock().unwrap() = Some(workspace);

    let NameReferent::UserProc(setup_mir_fn_id) = global_names.lookup("SETUP").unwrap() else {
        panic!("expected a user procedure");
    };
    let setup = functions[&mir_to_lir_fns[&setup_mir_fn_id]];
    let NameReferent::UserProc(go_mir_fn_id) = global_names.lookup("GO").unwrap() else {
        panic!("expected a user procedure");
    };
    let go = functions[&mir_to_lir_fns[&go_mir_fn_id]];
    let next_int = workspace.rng.clone();
    let mut ctx = ExecutionContext {
        workspace: &mut workspace,
        next_int,
        dirty_aggregator: DirtyAggregator::default(),
    };
    setup.call(&mut ctx, std::ptr::null_mut());
    visualize_update(ctx.workspace.world.generate_js_update_full());

    // let go_loop = async move {
    let mut workspace = workspace;
    let next_int = workspace.rng.clone();
    let mut ctx = ExecutionContext {
        workspace: &mut workspace,
        next_int,
        dirty_aggregator: DirtyAggregator::default(),
    };

    for _i in 1..1000 {
        go.call(&mut ctx, std::ptr::null_mut());
        visualize_update(ctx.workspace.world.generate_js_update_full());
    }
    // };
    // *GO_LOOP.lock().unwrap() = Some(Box::new(go_loop));
}

// extern "C" fn poll_go_loop() {
//     let workspace = WORKSPACE.lock().unwrap();
//     let Some(workspace) = workspace.as_mut() else {
//         return;
//     };
//     let next_int = workspace.rng.clone();
//     let mut ctx = ExecutionContext {
//         workspace: &mut workspace,
//         next_int,
//         dirty_aggregator: DirtyAggregator::default(),
//     };
//     go.call(&mut ctx, std::ptr::null_mut());
//     visualize_update(ctx.workspace.world.generate_js_update_full());
// }

struct ConsoleWriter;

impl io::Write for ConsoleWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        write_to_console(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct ConsoleWriterFactory;

impl<'a> MakeWriter<'a> for ConsoleWriterFactory {
    type Writer = ConsoleWriter;

    fn make_writer(&'a self) -> Self::Writer {
        ConsoleWriter
    }
}

pub use debug_print::{visualize_update, write_to_console, write_to_file};

#[cfg(not(target_arch = "wasm32"))]
mod debug_print {
    use std::{
        fs,
        io::{self, Write},
        path::Path,
    };

    pub fn write_to_console(buf: impl AsRef<[u8]>) {
        let buf = buf.as_ref();
        io::stdout().write_all(buf).unwrap();
    }

    pub fn write_to_file(filename: impl AsRef<Path>, buf: impl AsRef<[u8]>) {
        let filename = filename.as_ref();
        let buf = buf.as_ref();
        fs::write(filename, buf).unwrap();
    }

    pub fn visualize_update(update: impl AsRef<[u8]>) {
        write_to_file("update_visualize.txt", update.as_ref());
    }
}

#[cfg(target_arch = "wasm32")]
mod debug_print {
    use std::path::Path;

    pub fn write_to_console(buf: impl AsRef<[u8]>) {
        let buf = buf.as_ref();
        unsafe { r#extern::write_to_console(buf.as_ptr(), buf.len()) };
    }

    pub fn write_to_file(filename: impl AsRef<Path>, buf: impl AsRef<[u8]>) {
        let filename: &[u8] = filename.as_ref().as_os_str().as_encoded_bytes();
        let buf = buf.as_ref();
        unsafe {
            r#extern::write_to_file(filename.as_ptr(), filename.len(), buf.as_ptr(), buf.len())
        };
    }

    pub fn visualize_update(update: impl AsRef<[u8]>) {
        let update = update.as_ref();
        unsafe { r#extern::visualize_update(update.as_ptr(), update.len()) };
    }

    mod r#extern {
        unsafe extern "C" {
            pub fn write_to_console(message: *const u8, length: usize);

            pub fn write_to_file(
                name: *const u8,
                name_length: usize,
                bytes: *const u8,
                bytes_length: usize,
            );

            pub fn visualize_update(update: *const u8, length: usize);
        }
    }
}
