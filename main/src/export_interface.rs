use engine::{
    lir::{
        HostFunctionInfo,
        ValType::{F64, I32},
    },
    sim::value::PackedAny,
};

// TODO(mvp) the HostFunctionInfo should be automatically generated from the
// signatures of the actual host functions (probably done from the main crate
// rather than the engine crate).

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
