use engine::{
    lir::{
        HostFunctionInfo,
        ValType::{F64, I32, I64, Ptr},
    },
    sim::{
        patch::OptionPatchId,
        turtle::TurtleId,
        value::{NlFloat, PackedAny},
    },
    util::rng::{CanonRng, Rng as _},
    workspace::Workspace,
};

// TODO(mvp) the HostFunctionInfo should be automatically generated from the
// signatures of the actual host functions (probably done from the main crate
// rather than the engine crate).

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
