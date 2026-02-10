use std::collections::HashMap;

use engine::{
    exec::jit::{HostFunctionTable, InstallLir, InstallLirError, JitEntrypoint},
    lir,
};
use lir::HostFunction as Hf;

mod export_interface;
mod install_lir;

use export_interface::*;

#[derive(Default)]
pub struct LirInstaller {
    pub module_bytes: Vec<u8>,
}

impl InstallLir for LirInstaller {
    const HOST_FUNCTION_TABLE: HostFunctionTable = HostFunctionTable {
        clear_all: Hf(&CLEAR_ALL_INFO),
        reset_ticks: Hf(&RESET_TICKS_INFO),
        advance_tick: Hf(&ADVANCE_TICK_INFO),
        get_tick: Hf(&GET_TICK_INFO),
        create_turtles: Hf(&CREATE_TURTLES_INFO),
        ask_all_turtles: Hf(&ASK_ALL_TURTLES_INFO),
        ask_all_patches: Hf(&ASK_ALL_PATCHES_INFO),
        euclidean_distance_no_wrap: Hf(&EUCLIDEAN_DISTANCE_NO_WRAP_INFO),
        list_new: Hf(&LIST_NEW_INFO),
        list_push: Hf(&LIST_PUSH_INFO),
        one_of_list: Hf(&ONE_OF_LIST_INFO),
        scale_color: Hf(&SCALE_COLOR_INFO),
        rotate_turtle: Hf(&ROTATE_TURTLE_INFO),
        turtle_forward: Hf(&TURTLE_FORWARD_INFO),
        patch_at: Hf(&PATCH_AT_INFO),
        random_int: Hf(&RANDOM_INT_INFO),
        any_binary_op: Hf(&ANY_BINARY_OP_INFO),
        any_bool_binary_op: Hf(&ANY_BOOL_BINARY_OP_INFO),
        patch_ahead: Hf(&PATCH_AHEAD_INFO),
        patch_right_and_ahead: Hf(&PATCH_RIGHT_AND_AHEAD_INFO),
        diffuse_8_single_variable_buffer: Hf(&DIFFUSE_8_SINGLE_VARIABLE_BUFFER_INFO),
    };

    unsafe fn install_lir(
        &mut self,
        lir: &lir::Program,
    ) -> Result<HashMap<lir::FunctionId, JitEntrypoint>, InstallLirError> {
        let (result, module_bytes) = unsafe { install_lir::install_lir(lir) };
        self.module_bytes = module_bytes;
        result
    }
}
