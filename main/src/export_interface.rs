use engine::{
    exec::{helpers, jit::JitCallback},
    lir::{
        HostFunctionInfo,
        ValType::{F64, FnPtr, I32, I64, Ptr},
    },
    sim::{
        agent_schema::AgentFieldDescriptor,
        patch::{OptionPatchId, PatchId},
        topology::Point,
        turtle::{BreedId, TurtleId},
        value::{NlBox, NlFloat, NlList, PackedAny},
    },
    slotmap::KeyData,
    util::rng::{CanonRng, Rng as _},
    workspace::Workspace,
};

// TODO(mvp) the HostFunctionInfo should be automatically generated from the
// signatures of the actual host functions (probably done from the main crate
// rather than the engine crate).

pub static CLEAR_ALL_INFO: HostFunctionInfo = HostFunctionInfo {
    name: "oxitortoise_clear_all",
    parameter_types: &[Ptr],
    return_type: &[],
    addr: oxitortoise_clear_all as *const u8,
};
#[unsafe(no_mangle)]
pub extern "C" fn oxitortoise_clear_all(workspace: &mut Workspace) {
    workspace.world.clear_all();
}

pub static RESET_TICKS_INFO: HostFunctionInfo = HostFunctionInfo {
    name: "oxitortoise_reset_ticks",
    parameter_types: &[Ptr],
    return_type: &[],
    addr: oxitortoise_reset_ticks as *const u8,
};
#[unsafe(no_mangle)]
pub extern "C" fn oxitortoise_reset_ticks(workspace: &mut Workspace) {
    workspace.world.tick_counter.reset();
}

pub static GET_TICK_INFO: HostFunctionInfo = HostFunctionInfo {
    name: "oxitortoise_get_tick",
    parameter_types: &[Ptr],
    return_type: &[F64],
    addr: oxitortoise_get_tick as *const u8,
};
#[unsafe(no_mangle)]
pub extern "C" fn oxitortoise_get_tick(workspace: &mut Workspace) -> NlFloat {
    workspace.world.tick_counter.get().unwrap() // TODO(mvp) handle error
}

pub static ADVANCE_TICK_INFO: HostFunctionInfo = HostFunctionInfo {
    name: "oxitortoise_advance_tick",
    parameter_types: &[Ptr],
    return_type: &[],
    addr: oxitortoise_advance_tick as *const u8,
};
#[unsafe(no_mangle)]
pub extern "C" fn oxitortoise_advance_tick(workspace: &mut Workspace) {
    workspace.world.tick_counter.advance().unwrap(); // TODO(mvp) handle error
}

// NOTE: HostFunctionInfo had wrong name "reset_ticks", should be "create_turtles"
// NOTE: Signature mismatch - function takes breed, count, position, and callback, but HostFunctionInfo only has Ptr
pub static CREATE_TURTLES_INFO: HostFunctionInfo = HostFunctionInfo {
    name: "oxitortoise_create_turtles",
    parameter_types: &[Ptr, Ptr, I64, F64, Ptr, FnPtr],
    return_type: &[],
    addr: oxitortoise_create_turtles as *const u8,
};
#[unsafe(no_mangle)]
/// # Safety
///
/// The passed-in env and function pointer will be used to create a
/// [`JitCallback`] that needs to be live for the duration of this function
/// call. See [`JitCallback::new`] for the safety requirements on the caller.
pub unsafe extern "C" fn oxitortoise_create_turtles(
    workspace: &mut Workspace,
    rng: &mut CanonRng,
    breed: u64,
    count: NlFloat,
    birth_command_env: *mut u8,
    birth_command_fn_ptr: extern "C" fn(
        *mut u8,
        &mut Workspace,
        &mut CanonRng,
        u64, /* TurtleId */
    ) -> (),
) {
    let breed: BreedId = KeyData::from_ffi(breed).into();
    let position = Point { x: 0.0, y: 0.0 };
    // SAFETY: precondition
    let mut birth_command = unsafe { JitCallback::new(birth_command_env, birth_command_fn_ptr) };
    helpers::create_turtles(workspace, rng, breed, count, position, |workspace, rng, turtle_id| {
        birth_command.call_mut(workspace, rng, turtle_id.to_ffi())
    });
}

pub static ASK_ALL_TURTLES_INFO: HostFunctionInfo = HostFunctionInfo {
    name: "oxitortoise_for_all_turtles",
    parameter_types: &[Ptr, Ptr, Ptr, FnPtr],
    return_type: &[],
    addr: oxitortoise_for_all_turtles as *const u8,
};
/// # Safety
///
/// The passed-in env and function pointer will be used to create a
/// [`JitCallback`] that needs to be live for the duration of this function
/// call. See [`JitCallback::new`] for the safety requirements on the caller.
#[unsafe(no_mangle)]
pub extern "C" fn oxitortoise_for_all_turtles(
    workspace: &mut Workspace,
    rng: &mut CanonRng,
    block_env: *mut u8,
    block_fn_ptr: extern "C" fn(
        *mut u8,
        &mut Workspace,
        &mut CanonRng,
        u64, /* TurtleId */
    ) -> (),
) {
    // SAFETY: precondition
    let mut block = unsafe { JitCallback::new(block_env, block_fn_ptr) };
    helpers::for_all_turtles(workspace, rng, |workspace, rng, turtle_id| {
        block.call_mut(workspace, rng, turtle_id.to_ffi())
    });
}

pub static ASK_ALL_PATCHES_INFO: HostFunctionInfo = HostFunctionInfo {
    name: "oxitortoise_for_all_patches",
    parameter_types: &[Ptr, Ptr, Ptr, FnPtr],
    return_type: &[],
    addr: oxitortoise_for_all_patches as *const u8,
};
/// # Safety
///
/// The passed-in env and function pointer will be used to create a
/// [`JitCallback`] that needs to be live for the duration of this function
/// call. See [`JitCallback::new`] for the safety requirements on the caller.
#[unsafe(no_mangle)]
pub extern "C" fn oxitortoise_for_all_patches(
    workspace: &mut Workspace,
    rng: &mut CanonRng,
    block_env: *mut u8,
    block_fn_ptr: extern "C" fn(*mut u8, &mut Workspace, &mut CanonRng, PatchId) -> (),
) {
    // SAFETY: precondition
    let mut block = unsafe { JitCallback::new(block_env, block_fn_ptr) };
    helpers::for_all_patches(workspace, rng, |workspace, rng, patch_id| {
        block.call_mut(workspace, rng, patch_id)
    });
}

pub static EUCLIDEAN_DISTANCE_NO_WRAP_INFO: HostFunctionInfo = HostFunctionInfo {
    name: "oxitortoise_distance_euclidean_no_wrap",
    parameter_types: &[F64, F64, F64, F64],
    return_type: &[F64],
    addr: oxitortoise_distance_euclidean_no_wrap as *const u8,
};
#[unsafe(no_mangle)]
pub extern "C" fn oxitortoise_distance_euclidean_no_wrap(
    a_x: f64,
    a_y: f64,
    b_x: f64,
    b_y: f64,
) -> NlFloat {
    let a = Point { x: a_x, y: a_y };
    let b = Point { x: b_x, y: b_y };
    engine::sim::topology::euclidean_distance_no_wrap(a, b)
}

pub static PATCH_AT_INFO: HostFunctionInfo = HostFunctionInfo {
    name: "oxitortoise_patch_at",
    parameter_types: &[Ptr, F64, F64],
    return_type: &[I32],
    addr: oxitortoise_patch_at as *const u8,
};
#[unsafe(no_mangle)]
pub extern "C" fn oxitortoise_patch_at(
    workspace: &mut Workspace,
    point_x: f64,
    point_y: f64,
) -> PatchId {
    let point = Point { x: point_x, y: point_y };
    let point_int = point.round_to_int();
    workspace.world.topology.patch_at(point_int)
}

pub static ROTATE_TURTLE_INFO: HostFunctionInfo = HostFunctionInfo {
    name: "oxitortoise_rotate_turtle",
    parameter_types: &[Ptr, I64, F64],
    return_type: &[],
    addr: oxitortoise_rotate_turtle as *const u8,
};
#[unsafe(no_mangle)]
pub extern "C" fn oxitortoise_rotate_turtle(
    workspace: &mut Workspace,
    turtle_id: u64,
    angle: NlFloat,
) {
    let turtle_id = TurtleId::from_ffi(turtle_id);
    if let Some(heading) = workspace.world.turtles.get_turtle_heading_mut(turtle_id) {
        *heading += angle;
    }
}

pub static DIFFUSE_8_SINGLE_VARIABLE_BUFFER_INFO: HostFunctionInfo = HostFunctionInfo {
    name: "oxitortoise_diffuse_8",
    parameter_types: &[Ptr, I32, F64],
    return_type: &[],
    addr: oxitortoise_diffuse_8 as *const u8,
};
#[unsafe(no_mangle)]
pub extern "C" fn oxitortoise_diffuse_8(
    workspace: &mut Workspace,
    field: u32, /* AgentFieldDescriptor */
    diffusion_rate: NlFloat,
) {
    let field = AgentFieldDescriptor::from_u16(field as u16);
    engine::sim::topology::diffuse::diffuse_8_single_variable_buffer(
        &mut workspace.world,
        field,
        diffusion_rate,
    );
}

pub static SCALE_COLOR_INFO: HostFunctionInfo = HostFunctionInfo {
    name: "oxitortoise_scale_color",
    parameter_types: &[F64, F64, F64, F64],
    return_type: &[F64],
    addr: oxitortoise_scale_color as *const u8,
};
#[unsafe(no_mangle)]
pub extern "C" fn oxitortoise_scale_color(
    color: NlFloat,
    value: NlFloat,
    min: NlFloat,
    max: NlFloat,
) -> NlFloat {
    engine::sim::color::scale_color(color.into(), value, min, max).into()
}

pub static RANDOM_INT_INFO: HostFunctionInfo = HostFunctionInfo {
    name: "oxitortoise_random_int",
    parameter_types: &[Ptr, F64],
    return_type: &[F64],
    addr: oxitortoise_random_int as *const u8,
};
#[unsafe(no_mangle)]
pub extern "C" fn oxitortoise_random_int(rng: &mut CanonRng, max: f64) -> f64 {
    // TODO move the casting logic to the engine, not in this shim
    rng.next_int(max as i64) as f64
}

pub static TURTLE_FORWARD_INFO: HostFunctionInfo = HostFunctionInfo {
    name: "oxitortoise_turtle_forward",
    parameter_types: &[Ptr, I64, F64],
    return_type: &[],
    addr: oxitortoise_turtle_forward as *const u8,
};
#[unsafe(no_mangle)]
pub extern "C" fn oxitortoise_turtle_forward(
    workspace: &mut Workspace,
    turtle_id: u64,
    distance: NlFloat,
) {
    // TODO: this is a lot of logic. it should be in the engine
    let world = &mut workspace.world;
    let turtle_id = TurtleId::from_ffi(turtle_id);
    let heading = world.turtles.get_turtle_heading(turtle_id).unwrap();
    let position = world.turtles.get_turtle_position(turtle_id).unwrap();
    let new_pos = world.topology.offset_distance_by_heading(*position, *heading, distance);
    if let Some(new_pos) = new_pos {
        *world.turtles.get_turtle_position_mut(turtle_id).unwrap() = new_pos;
    }
}

pub static LIST_NEW_INFO: HostFunctionInfo = HostFunctionInfo {
    name: "oxitortoise_list_new",
    parameter_types: &[],
    return_type: &[Ptr],
    addr: oxitortoise_list_new as *const u8,
};
#[unsafe(no_mangle)]
pub extern "C" fn oxitortoise_list_new() -> NlBox<NlList> {
    let list = NlList::new();
    NlBox::new(list)
}

pub static LIST_PUSH_INFO: HostFunctionInfo = HostFunctionInfo {
    name: "oxitortoise_list_push",
    parameter_types: &[Ptr, F64],
    return_type: &[Ptr],
    addr: oxitortoise_list_push as *const u8,
};
#[unsafe(no_mangle)]
pub extern "C" fn oxitortoise_list_push(
    mut list: NlBox<NlList>,
    element: PackedAny,
) -> NlBox<NlList> {
    list.push(element);
    list
}

pub static ONE_OF_LIST_INFO: HostFunctionInfo = HostFunctionInfo {
    name: "oxitortoise_one_of_list",
    parameter_types: &[Ptr, Ptr],
    return_type: &[F64],
    addr: oxitortoise_one_of_list as *const u8,
};
#[unsafe(no_mangle)]
pub extern "C" fn oxitortoise_one_of_list(
    rng: &mut CanonRng,
    mut list: NlBox<NlList>,
) -> PackedAny {
    let index = rng.next_int(list.len() as i64) as usize; // TODO casts okay?
    list.swap_remove(index)
}

// TODO: Write function definition for patch_ahead
pub static PATCH_AHEAD_INFO: HostFunctionInfo = HostFunctionInfo {
    name: "oxitortoise_patch_ahead",
    parameter_types: &[Ptr, I64, F64],
    return_type: &[I32],
    addr: oxitortoise_patch_ahead as *const u8,
};
#[unsafe(no_mangle)]
pub extern "C" fn oxitortoise_patch_ahead(
    workspace: &mut Workspace,
    turtle_id: u64,
    distance: NlFloat,
) -> OptionPatchId {
    // TODO: this is a lot of logic. it should be in the engine
    let world = &mut workspace.world;
    let turtle_id = TurtleId::from_ffi(turtle_id);
    let heading = world.turtles.get_turtle_heading(turtle_id).unwrap();
    let position = world.turtles.get_turtle_position(turtle_id).unwrap();
    let pos_ahead = world.topology.offset_distance_by_heading(*position, *heading, distance);
    if let Some(pos_ahead) = pos_ahead {
        world.topology.patch_at(pos_ahead.round_to_int()).into()
    } else {
        OptionPatchId::NOBODY
    }
}

pub static PATCH_RIGHT_AND_AHEAD_INFO: HostFunctionInfo = HostFunctionInfo {
    name: "oxitortoise_patch_right_and_ahead",
    parameter_types: &[Ptr, I64, F64, F64],
    return_type: &[I32],
    addr: oxitortoise_patch_right_and_ahead as *const u8,
};
#[unsafe(no_mangle)]
pub extern "C" fn oxitortoise_patch_right_and_ahead(
    workspace: &mut Workspace,
    turtle_id: u64,
    angle: NlFloat,
    distance: NlFloat,
) -> OptionPatchId {
    // TODO: this is a lot of logic. it should be in the engine
    let world = &mut workspace.world;
    let turtle_id = TurtleId::from_ffi(turtle_id);
    let heading = world.turtles.get_turtle_heading(turtle_id).unwrap();
    let heading_right = *heading + angle;
    let position = world.turtles.get_turtle_position(turtle_id).unwrap();
    let pos_ahead = world.topology.offset_distance_by_heading(*position, heading_right, distance);
    if let Some(pos_ahead) = pos_ahead {
        world.topology.patch_at(pos_ahead.round_to_int()).into()
    } else {
        OptionPatchId::NOBODY
    }
}

// TODO: Write function definition for any_binary_op
pub static ANY_BINARY_OP_INFO: HostFunctionInfo = HostFunctionInfo {
    name: "oxitortoise_any_binary_op",
    parameter_types: &[F64, F64, I32],
    return_type: &[F64],
    addr: oxitortoise_any_binary_op as *const u8,
};
#[unsafe(no_mangle)]
pub extern "C" fn oxitortoise_any_binary_op(
    _lhs: PackedAny,
    _rhs: PackedAny,
    _op: u32,
) -> PackedAny {
    todo!()
    // let op = BinaryOpcode::try_from(op as u8).unwrap();
    // match op {
    //     BinaryOpcode::Add => lhs + rhs,
    //     BinaryOpcode::Sub => lhs - rhs,
    //     BinaryOpcode::Mul => lhs * rhs,
    //     BinaryOpcode::Div => lhs / rhs,
    //     _ => unimplemented!("unsupported binary operation: {:?}", op),
    // }
}

pub static ANY_BOOL_BINARY_OP_INFO: HostFunctionInfo = HostFunctionInfo {
    name: "oxitortoise_any_bool_binary_op",
    parameter_types: &[F64, F64, I32],
    return_type: &[I32],
    addr: oxitortoise_any_bool_binary_op as *const u8,
};

#[unsafe(no_mangle)]
pub extern "C" fn oxitortoise_any_bool_binary_op(
    _lhs: PackedAny,
    _rhs: PackedAny,
    _op: u32,
) -> bool {
    todo!()
    // let op = BinaryOpcode::try_from(op as u8).unwrap();
    // match op {
    //     BinaryOpcode::Eq => lhs == rhs,
    //     BinaryOpcode::Neq => lhs != rhs,
    //     BinaryOpcode::Lt => lhs < rhs,
    //     BinaryOpcode::Lte => lhs <= rhs,
    //     BinaryOpcode::Gt => lhs > rhs,
    //     BinaryOpcode::Gte => lhs >= rhs,
    //     BinaryOpcode::And => lhs.and(rhs),
    //     BinaryOpcode::Or => lhs.or(rhs),
    //     _ => unimplemented!("unsupported binary operation: {:?}", op),
    // }
}
