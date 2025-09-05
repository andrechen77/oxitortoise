use std::rc::Rc;

use engine::{
    exec::{
        CanonExecutionContext,
        jit::{InstallLir as _, JitCallback},
    },
    lir::{self, lir_function},
    mir,
    sim::{
        agent_schema::{PatchSchema, TurtleSchema},
        patch::{OFFSET_PATCHES_TO_DATA, PatchBaseData, PatchId, Patches},
        shapes::Shapes,
        tick::Tick,
        topology::{
            OFFSET_TOPOLOGY_TO_MAX_PXCOR, OFFSET_TOPOLOGY_TO_MAX_PYCOR, Point, Topology,
            TopologySpec,
        },
        turtle::{
            Breed, BreedId, OFFSET_TURTLES_TO_DATA, TurtleBaseData, TurtleId, TurtleVarDesc,
            Turtles,
        },
        value::{NetlogoInternalType, UnpackedDynBox},
        world::World,
    },
    slotmap::{Key as _, SlotMap},
    updater::DirtyAggregator,
    util::{cell::RefCell, rng::CanonRng, row_buffer::RowBuffer},
    workspace::Workspace,
};
use oxitortoise_main::LirInstaller;

#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    fn write_to_console(message: *const u8, length: usize);
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

// static mut CONTEXT: Option<CanonExecutionContext> = None;
// static mut SETUP_FN: Option<JitEntry> = None;
// static mut GO_FN: Option<JitEntry> = None;

pub fn main() {
    real_print("Hello, world!");

    let (_workspace, default_turtle_breed) = create_workspace();

    // dynamically load the model_code
    let lir = create_lir(default_turtle_breed);
    let _entrypoints = unsafe { LirInstaller::install_lir(&lir).unwrap() };

    // real_print("installed new functions");
    // // SAFETY: no one else has a reference to this variable since this is called
    // // in single-threaded contexts only and this function is the entry point.
    // unsafe {
    //     GO_FN = Some(new_functions.remove(1));
    //     SETUP_FN = Some(new_functions.remove(0));
    // }

    // // create the workspace and execution context
    // let workspace = Box::leak(Box::new(create_workspace()));
    // let mut dirty_aggregator = DirtyAggregator::new();
    // dirty_aggregator.world |= FlagSet::full();
    // dirty_aggregator.tick = workspace.world.tick_counter.clone();
    // dirty_aggregator.reserve_turtles(1000);
    // dirty_aggregator.reserve_patches(10000);
    // // real_print(format!("Updater: {:?}", &updater));
    // let rng = workspace.rng.clone();
    // let context = CanonExecutionContext { workspace, dirty_aggregator, next_int: rng };
    // // SAFETY: no one else has a reference to this variable since this is called
    // // in single-threaded contexts only and this function is the entry point.
    // unsafe {
    //     CONTEXT = Some(context);
    // }
}

// #[no_mangle]
// #[allow(static_mut_refs)]
// pub fn run_setup_and_go() {
//     // SAFETY: no one else has a reference to this variable since this is called
//     // in single-threaded contexts only and this function is the entry point.
//     let context = unsafe { CONTEXT.as_mut().unwrap() };
//     let setup_fn = unsafe { SETUP_FN.as_ref().unwrap() };
//     let go_fn = unsafe { GO_FN.as_ref().unwrap() };
//     // call the dynamically loaded functions
//     setup_fn.call(context, std::ptr::null_mut());
//     // real_print(format!("After setup: {:?}", &context.updater));
//     for _ in 0..1000 {
//         go_fn.call(context, std::ptr::null_mut());
//     }
//     // real_print(format!("After go: {:?}", &context.updater));
// }

// #[no_mangle]
// #[allow(static_mut_refs)]
// pub fn run_diy() {
//     let context = unsafe { CONTEXT.as_mut().unwrap() };
//     dynamic_link::oxitortoise_clear_all(context);
//     extern "C" fn body0(_env: *mut u8, context: &mut CanonExecutionContext, turtle_id: u64) {
//         let turtle_id = TurtleId::from_ffi(turtle_id);
//         let turtle_data =
//             context.workspace.world.turtles.get_turtle_base_data_mut(turtle_id).unwrap();
//         turtle_data.color = Float::new(15.0).into();
//         turtle_data.size = Float::new(2.0);
//         context.dirty_aggregator.get_turtles_mut()[turtle_id.index()] |= TurtleProp::Color;
//     }
//     let breed = dynamic_link::oxitortoise_get_default_turtle_breed(context);
//     dynamic_link::oxitortoise_create_turtles(
//         context,
//         breed,
//         125,
//         Point { x: 0.0, y: 0.0 },
//         JitCallback { fn_ptr: body0, env: std::ptr::null_mut() },
//     )
// }

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

fn create_lir(default_turtle_breed: BreedId) -> lir::Program {
    use std::mem::offset_of;

    let mut lir = lir::Program::default();
    let mut lir_fn_tracker = SlotMap::with_key();
    let fn_clear_all = lir.host_functions.push_and_get_key(lir::HostFunction {
        name: "oxitortoise_clear_all",
        parameter_types: vec![lir::ValType::Ptr],
        return_type: vec![],
    });
    let fn_create_turtles = lir.host_functions.push_and_get_key(lir::HostFunction {
        name: "oxitortoise_create_turtles",
        parameter_types: vec![
            lir::ValType::Ptr,
            lir::ValType::I32,
            lir::ValType::I32,
            lir::ValType::Ptr,
            lir::ValType::Ptr,
        ],
        return_type: vec![],
    });
    let fn_for_all_patches = lir.host_functions.push_and_get_key(lir::HostFunction {
        name: "oxitortoise_for_all_patches",
        parameter_types: vec![lir::ValType::Ptr, lir::ValType::Ptr],
        return_type: vec![],
    });
    let fn_distance_euclidean_no_wrap = lir.host_functions.push_and_get_key(lir::HostFunction {
        name: "oxitortoise_distance_euclidean_no_wrap",
        parameter_types: vec![
            lir::ValType::F64,
            lir::ValType::F64,
            lir::ValType::F64,
            lir::ValType::F64,
        ],
        return_type: vec![lir::ValType::F64],
    });
    let fn_next_int = lir.host_functions.push_and_get_key(lir::HostFunction {
        name: "oxitortoise_next_int",
        parameter_types: vec![lir::ValType::I32, lir::ValType::I32],
        return_type: vec![lir::ValType::I32],
    });
    let fn_reset_ticks = lir.host_functions.push_and_get_key(lir::HostFunction {
        name: "oxitortoise_reset_ticks",
        parameter_types: vec![lir::ValType::Ptr],
        return_type: vec![],
    });
    let fn_get_ticks = lir.host_functions.push_and_get_key(lir::HostFunction {
        name: "oxitortoise_get_ticks",
        parameter_types: vec![lir::ValType::Ptr],
        return_type: vec![lir::ValType::F64],
    });

    // TODO put the right values for these
    let stride_of_turtle_data = 10;
    let stride_of_patch_data0 = 10;
    let offset_patch_to_nest = 0;
    let offset_patch_to_nest_scent = 0;
    let offset_patch_to_food_source_number = 0;
    let offset_patch_to_food = 0;

    lir_function! {
        fn setup_body_0(Ptr _env, Ptr ctx, I64 next_turtle) -> [],
        vars: [],
        stack_space: 0,
        main: {
            [turtle_idx] = I64ToI32(var_load(next_turtle));

            [workspace] = mem_load(Ptr, offset_of!(CanonExecutionContext, workspace))(var_load(ctx));
            [turtle_buffer] = mem_load(
                Ptr,
                offset_of!(Workspace, world)
                    + offset_of!(World, turtles)
                    + OFFSET_TURTLES_TO_DATA
                    + 0 * size_of::<Option<RowBuffer>>()
            )(workspace);
            [base_data] = derive_element(stride_of_turtle_data)(turtle_buffer, turtle_idx);

            mem_store(offset_of!(TurtleBaseData, size))(base_data, constant(F64, 2.0f64.to_bits()));
            mem_store(offset_of!(TurtleBaseData, color))(base_data, constant(F64, 15.0f64.to_bits()));
        }
    }
    let setup_body_0_key = lir_fn_tracker.insert(());
    lir.user_functions.insert(setup_body_0_key, setup_body_0);

    lir_function! {
        fn recolor_patch(Ptr _env, I32 _next_patch) -> [],
        vars: [],
        stack_space: 0,
        main: {
        }
    }
    let recolor_patch_key = lir_fn_tracker.insert(());
    lir.user_functions.insert(recolor_patch_key, recolor_patch);

    lir_function! {
        fn setup_body_1(Ptr _env, Ptr ctx, I32 next_patch) -> [],
        vars: [],
        stack_space: 0,
        main: {
            // calculate distancexy 0 0

            [workspace] = mem_load(Ptr, offset_of!(CanonExecutionContext, workspace))(var_load(ctx));
            [patch_buffer] = mem_load(
                Ptr,
                offset_of!(Workspace, world)
                    + offset_of!(World, patches)
                    + OFFSET_PATCHES_TO_DATA
                    + 0 * size_of::<Option<RowBuffer>>()
            )(workspace);
            [base_data] = derive_element(stride_of_patch_data0)(patch_buffer, var_load(next_patch));
            [pos_x] = mem_load(F64, offset_of!(PatchBaseData, position) + offset_of!(Point, x))(base_data);
            [pos_y] = mem_load(F64, offset_of!(PatchBaseData, position) + offset_of!(Point, y))(base_data);
            [distance] = call_host_fn(fn_distance_euclidean_no_wrap -> [F64])(
                pos_x,
                pos_y,
                constant(F64, 0.0f64.to_bits()),
                constant(F64, 0.0f64.to_bits()),
            );

            // set nest? (distancexy 0 0) < 5
            mem_store(offset_patch_to_nest)(base_data, FLt(distance, constant(F64, 5.0f64.to_bits())));

            // set next-scent 200 - distancexy 0 0
            mem_store(offset_patch_to_nest_scent)(base_data, FSub(
                constant(F64, 200.0f64.to_bits()),
                distance,
            ));

            [max_pxcor] = mem_load(F64, offset_of!(Workspace, world) + offset_of!(World, topology) + OFFSET_TOPOLOGY_TO_MAX_PXCOR)(workspace);
            [max_pycor] = mem_load(F64, offset_of!(Workspace, world) + offset_of!(World, topology) + OFFSET_TOPOLOGY_TO_MAX_PYCOR)(workspace);

            // if (distancexy (0.6 * max-pxcor) 0) < 5 [ set food-source-number 1 ]
            [distance] = call_host_fn(fn_distance_euclidean_no_wrap -> [F64])(
                pos_x,
                pos_y,
                FMul(constant(F64, 0.6f64.to_bits()), max_pxcor),
                constant(F64, 0.0f64.to_bits()),
            );
            [] = if_else(-> [])(FLt(distance, constant(F64, 5.0f64.to_bits()))) then: {
                mem_store(offset_patch_to_food_source_number)(base_data, constant(F64, 1.0f64.to_bits()));
            } else_: {};

            // if (distancexy (-0.6 * max-pxcor) (-0.6 * max-pycor)) < 5 [ set food-source-number 2 ]
            [distance] = call_host_fn(fn_distance_euclidean_no_wrap -> [F64])(
                pos_x,
                pos_y,
                FMul(constant(F64, (-0.6f64).to_bits()), max_pxcor),
                FMul(constant(F64, (-0.6f64).to_bits()), max_pycor),
            );
            [] = if_else(-> [])(FLt(distance, constant(F64, 5.0f64.to_bits()))) then: {
                mem_store(offset_patch_to_food_source_number)(base_data, constant(F64, 2.0f64.to_bits()));
            } else_: {};

            // if (distancexy (-0.8 * max-pxcor) (0.8 * max-pycor)) < 5 [ set food-source-number 3 ]
            [distance] = call_host_fn(fn_distance_euclidean_no_wrap -> [F64])(
                pos_x,
                pos_y,
                FMul(constant(F64, (-0.8f64).to_bits()), max_pxcor),
                FMul(constant(F64, 0.8f64.to_bits()), max_pycor),
            );
            [] = if_else(-> [])(FLt(distance, constant(F64, 5.0f64.to_bits()))) then: {
                mem_store(offset_patch_to_food_source_number)(base_data, constant(F64, 3.0f64.to_bits()));
            } else_: {};

            // if food-source-number > 0 [ set food one-of [1 2] ]
            [food_source_number] = mem_load(F64, offset_patch_to_food_source_number)(base_data);
            [] = if_else(-> [])(FGt(food_source_number, constant(F64, 0.0f64.to_bits()))) then_0: {
                [rand_index] = call_host_fn(fn_next_int -> [I32])(var_load(ctx), constant(I32, 2));
                [food] = if_else(-> [F64])(rand_index) then_1: {
                    break_(then_1)(constant(F64, 1.0f64.to_bits()));
                } else_1: {
                    break_(else_1)(constant(F64, 2.0f64.to_bits()));
                };
                mem_store(offset_patch_to_food)(base_data, food);
            } else_0: {};

            // recolor-patch
            [] = call_user_function(recolor_patch_key -> [])(var_load(ctx), var_load(next_patch));
        }
    }
    let setup_body_1_key = lir_fn_tracker.insert(());
    lir.user_functions.insert(setup_body_1_key, setup_body_1);

    lir_function! {
        fn setup(I32 ctx) -> [],
        vars: [],
        stack_space: 32,
        main: {
            [workspace] = mem_load(Ptr, offset_of!(CanonExecutionContext, workspace))(var_load(ctx));
            [world] = derive_field(offset_of!(Workspace, world))(workspace);

            // clear-all

            [] = call_host_fn(fn_clear_all -> [])(var_load(ctx));

            // create-turtles

            [default_turtle_breed] = constant(I32, default_turtle_breed.data().as_ffi());
            // at stack offset 0, create a point (0, 0)
            stack_store(offset_of!(Point, x))(constant(F64, 0.0f64.to_bits()));
            stack_store(offset_of!(Point, y))(constant(F64, 0.0f64.to_bits()));
            // at stack offset 16, create a closure
            stack_store(16 + offset_of!(JitCallback<TurtleId, ()>, fn_ptr))(user_fn_ptr(setup_body_0_key));
            // don't bother storing anything in env since it's not used.
            [] = call_host_fn(fn_create_turtles -> [])(
                var_load(ctx),
                default_turtle_breed,
                constant(I32, 125),
                stack_addr(0),
                stack_addr(16),
            );

            // setup-patches

            // at stack offset 16, create a closure
            stack_store(16 + offset_of!(JitCallback<PatchId, ()>, fn_ptr))(user_fn_ptr(setup_body_1_key));
            // don't bother storing anything in env since it's not used
            [] = call_host_fn(fn_for_all_patches -> [])(
                var_load(ctx),
                stack_addr(16),
            );

            [] = call_host_fn(fn_reset_ticks -> [])(world);

            [ticks] = call_host_fn(fn_get_ticks -> [F64])(world);
            mem_store(
                offset_of!(CanonExecutionContext, dirty_aggregator)
                + offset_of!(DirtyAggregator, tick)
            )(var_load(ctx), ticks);
        }
    }
    let setup_key = lir_fn_tracker.insert(());
    lir.user_functions.insert(setup_key, setup);
    lir.entrypoints.push(setup_key);

    lir
}

fn create_mir(default_turtle_breed: BreedId) -> mir::Program {
    let mut program = mir::Program::default();
    use mir::node;

    let setup_body_0 = {
        let mut locals = SlotMap::with_key();
        let local_env = locals.insert(mir::LocalDeclaration {
            debug_name: Some("env".to_string()),
            ty: NetlogoInternalType::UNTYPED_PTR,
            storage: mir::LocalStorage::Register,
        });
        let local_context = locals.insert(mir::LocalDeclaration {
            debug_name: Some("context".to_string()),
            ty: NetlogoInternalType::UNTYPED_PTR,
            storage: mir::LocalStorage::Register,
        });
        let local_self = locals.insert(mir::LocalDeclaration {
            debug_name: Some("self".to_string()),
            ty: NetlogoInternalType::TURTLE_ID,
            storage: mir::LocalStorage::Register,
        });
        let mut nodes: SlotMap<mir::NodeId, Box<dyn mir::EffectfulNode>> = SlotMap::with_key();
        let statements = vec![
            mir::StatementKind::Node({
                let context = nodes.insert(Box::new(node::GetLocalVar { local_id: local_context }));
                let turtle = nodes.insert(Box::new(node::GetLocalVar { local_id: local_self }));
                let value =
                    nodes.insert(Box::new(node::Constant { value: UnpackedDynBox::Float(2.0) }));
                nodes.insert(Box::new(node::SetTurtleVar {
                    context,
                    turtle,
                    var: TurtleVarDesc::Size,
                    value,
                }))
            }),
            mir::StatementKind::Node({
                let context = nodes.insert(Box::new(node::GetLocalVar { local_id: local_context }));
                let turtle = nodes.insert(Box::new(node::GetLocalVar { local_id: local_self }));
                let value =
                    nodes.insert(Box::new(node::Constant { value: UnpackedDynBox::Float(15.0) }));
                nodes.insert(Box::new(node::SetTurtleVar {
                    context,
                    turtle,
                    var: TurtleVarDesc::Color,
                    value,
                }))
            }),
        ];
        mir::Function {
            debug_name: Some("setup/body_0".to_string()),
            parameters: vec![local_env, local_context, local_self],
            return_ty: None,
            locals,
            nodes,
            cfg: mir::StatementBlock { statements },
        }
    };
    let setup_body_0 = program.functions.insert(setup_body_0);

    let setup = {
        let mut locals = SlotMap::with_key();
        let local_context = locals.insert(mir::LocalDeclaration {
            debug_name: Some("context".to_string()),
            ty: NetlogoInternalType::UNTYPED_PTR,
            storage: mir::LocalStorage::Register,
        });
        let mut nodes: SlotMap<mir::NodeId, Box<dyn mir::EffectfulNode>> = SlotMap::with_key();
        let statements = vec![
            mir::StatementKind::Node({
                let ctx = nodes.insert(Box::new(node::GetLocalVar { local_id: local_context }));
                nodes.insert(Box::new(node::ClearAll { context: ctx }))
            }),
            mir::StatementKind::Node({
                let ctx = nodes.insert(Box::new(node::GetLocalVar { local_id: local_context }));
                let num_turtles =
                    nodes.insert(Box::new(node::Constant { value: UnpackedDynBox::Int(125) }));
                let body =
                    nodes.insert(Box::new(node::Closure { captures: vec![], body: setup_body_0 }));
                nodes.insert(Box::new(node::CreateTurtles {
                    context: ctx,
                    breed: default_turtle_breed,
                    num_turtles,
                    body,
                }))
            }),
        ];
        mir::Function {
            debug_name: Some("setup".to_string()),
            parameters: vec![local_context],
            return_ty: None,
            locals,
            nodes,
            cfg: mir::StatementBlock { statements },
        }
    };
    let setup = program.functions.insert(setup);

    program
}

#[cfg(test)]
mod tests {
    use engine::mir::lowering::lower;

    use super::*;

    #[test]
    fn lower_mir() {
        let (workspace, default_turtle_breed) = create_workspace();
        let mut mir = create_mir(default_turtle_breed);

        let (_, setup_body_0) = mir
            .functions
            .iter_mut()
            .find(|(k, v)| v.debug_name == Some("setup/body_0".to_string()))
            .unwrap();
        let dot_string = setup_body_0.to_dot_string();
        std::fs::write("setup_body_0.dot", dot_string).expect("Failed to write dot file");
        lower(setup_body_0, &workspace);
        let dot_string = setup_body_0.to_dot_string();
        std::fs::write("setup_body_0_lowered.dot", dot_string).expect("Failed to write dot file");

        let (_, setup) = mir
            .functions
            .iter_mut()
            .find(|(k, v)| v.debug_name == Some("setup".to_string()))
            .unwrap();
        let dot_string = setup.to_dot_string();
        std::fs::write("setup.dot", dot_string).expect("Failed to write dot file");
        lower(setup, &workspace);
        let dot_string = setup.to_dot_string();
        std::fs::write("setup_lowered.dot", dot_string).expect("Failed to write dot file");

        let lir = mir::mir_to_lir(&mir, &workspace);
        std::fs::write("lir", format!("{}", lir)).expect("Failed to write lir file");

        let _entrypoints = unsafe { LirInstaller::install_lir(&lir).unwrap() };
    }
}
