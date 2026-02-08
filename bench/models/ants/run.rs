use std::{cell::RefCell, io, rc::Rc};

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
        patch::{PatchSchema, Patches},
        shapes::Shapes,
        tick::Tick,
        topology::{Topology, TopologySpec},
        turtle::{Breed, BreedId, TurtleSchema, Turtles},
        world::World,
    },
    slotmap::SecondaryMap,
    updater::DirtyAggregator,
    util::rng::CanonRng,
    workspace::Workspace,
};
use oxitortoise_main::LirInstaller;
use tracing::{Level, error, info};
use tracing_subscriber::{
    Layer as _, filter::Targets, fmt::MakeWriter, layer::SubscriberExt as _,
    util::SubscriberInitExt as _,
};

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
        rng: Rc::new(RefCell::new(CanonRng::new(0))),
    }
}

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

    let TurtleBreeds::Full(breeds) = program.turtle_breeds else {
        panic!("turtle breeds are not full");
    };

    // set up the workspace
    let mut workspace = create_workspace(
        program.globals_schema.unwrap(),
        program.turtle_schema.unwrap(),
        breeds,
        program.patch_schema.unwrap(),
    );

    let NameReferent::UserProc(mir_fn_id) = global_names.lookup("SETUP").unwrap() else {
        panic!("expected a user procedure");
    };
    let setup = functions[&mir_to_lir_fns[&mir_fn_id]];

    let next_int = workspace.rng.clone();
    let mut execution_context = ExecutionContext {
        workspace: &mut workspace,
        next_int,
        dirty_aggregator: DirtyAggregator::default(),
    };
    setup.call(&mut execution_context, std::ptr::null_mut());
}

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

pub use debug_print::{write_to_console, write_to_file};

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

    mod r#extern {
        unsafe extern "C" {
            pub fn write_to_console(message: *const u8, length: usize);

            pub fn write_to_file(
                name: *const u8,
                name_length: usize,
                bytes: *const u8,
                bytes_length: usize,
            );
        }
    }
}
