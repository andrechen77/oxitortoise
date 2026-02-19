use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    sync::{LazyLock, Mutex},
};

use crate::export_interface::*;

use engine::{
    exec::{
        ExecutionContext,
        jit::{HostFunctionTable, InstallLir, InstallLirError, InstalledObj, JitEntrypoint},
    },
    lir,
    sim::value::PackedAny,
};
use lir::HostFunction as Hf;

#[derive(Default)]
pub struct LirInstaller {
    pub module_bytes: Vec<u8>,
}

impl InstallLir for LirInstaller {
    type Obj = Obj;

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

    unsafe fn install_lir(&mut self, lir: &lir::Program) -> Result<Self::Obj, InstallLirError> {
        let (result, module_bytes) = unsafe { install_lir(lir) };
        self.module_bytes = module_bytes;
        result
    }
}

static FUNCTION_INSTALLER: LazyLock<Mutex<FunctionInstaller>> = LazyLock::new(|| {
    // SAFETY: the function installer is the only code that interacts with the
    // instantiation APIs. Since this is called in a LazyLock, it is only
    // instantiated once. Therefore, it is safe to assume that no other code is
    // interacting with the instantiation APIs.
    Mutex::new(unsafe { FunctionInstaller::new() })
});

pub unsafe fn install_lir(lir: &lir::Program) -> (Result<Obj, InstallLirError>, Vec<u8>) {
    unsafe {
        let Ok(mut installer) = FUNCTION_INSTALLER.lock() else {
            return (Err(InstallLirError::InstallerPoisoned), vec![]);
        };
        installer.install_lir(lir)
    }
}

#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    /// Instantiates the module at the specified bytes (creating an "auxiliary
    /// instance"). All exports of the currently running instance will be
    /// imported by the auxiliary instance under the "main_module" namespace.
    /// This includes:
    /// - memory "memory": the memory used by the main module. The auxiliary
    ///   instance will be able to read and write our memory.
    /// - table "__indirect_function_table": the function table used by the main
    ///   module. The auxiliary instance will be able to install its own
    ///   functions into this function table, which we will be able to call.
    /// - global "__stack_pointer": the stack pointer used by the main module.
    ///   The auxiliary instance will use this stack to store its own local
    ///   variables.
    /// - all function exports. The auxiliary instance will be able to call our
    ///   functions.
    ///
    /// # Safety
    ///
    /// Since the auxiliary instance will be executed with the current
    /// instance's memory and function table, so must not cause any undefined
    /// behavior.
    fn instantiate_module(module_bytes_start: *const u8, module_bytes_len: usize) -> bool;

    // TODO(wishlist) this shouldn't have to be imported from the host
    // environment. it would be ideal if we could do this by emitting Wasm
    // instructions in the current module directly to just grow the table.
    /// Creates new slots in the function table by growing the table, and
    /// returns the index of the first new slot in the table. Returns `usize::MAX`
    /// if the operation could not be completed.
    safe fn grow_function_table(num_slots: usize) -> usize;
}

#[cfg(not(target_arch = "wasm32"))]
fn grow_function_table(num_slots: usize) -> usize {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT_FREE_SLOT: AtomicUsize = AtomicUsize::new(420);
    NEXT_FREE_SLOT.fetch_add(num_slots, Ordering::Relaxed)
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn instantiate_module(module_bytes_start: *const u8, module_bytes_len: usize) -> bool {
    let module_bytes = unsafe { std::slice::from_raw_parts(module_bytes_start, module_bytes_len) };
    std::fs::write("test.wasm", module_bytes).unwrap();
    panic!("Module bytes written to test.wasm");
}

/// Tracks state required to install additional functions to the current
/// instance's function table.
struct FunctionInstaller {
    free_slots: BinaryHeap<Reverse<usize>>,
}

impl FunctionInstaller {
    /// # Safety
    ///
    /// A `FunctionInstaller` assumes that it is the only one interacting with
    /// the imported instantiation APIs. Therefore, a FunctionInstaller cannot
    /// exist at any time other code might be attempting to make those
    /// interactions, including when other `FunctionInstaller`s exist.
    unsafe fn new() -> Self {
        Self { free_slots: BinaryHeap::new() }
    }

    /// Installs the specified LIR program into the current instance. Potential
    /// callbacks and entrypoints in the LIR program are also installed in the
    /// function table, and must have the correct signature.
    ///
    /// # Safety
    ///
    /// The LIR program being installed will use the same namespace and
    /// indirect function table as the current instance. The code must not
    /// cause undefined behavior.
    unsafe fn install_lir(
        &mut self,
        lir: &lir::Program,
    ) -> (Result<Obj, InstallLirError>, Vec<u8>) {
        struct A<'a>(&'a mut BinaryHeap<Reverse<usize>>);
        impl<'a> lir_to_wasm::FnTableSlotAllocator for A<'a> {
            fn allocate_slot(&mut self) -> usize {
                if let Some(Reverse(slot)) = self.0.pop() {
                    return slot;
                }

                let num_new_slots = 32;
                let first_new_slot = grow_function_table(num_new_slots);
                let new_slots = (first_new_slot..(first_new_slot + num_new_slots)).map(Reverse);
                self.0.extend(new_slots);

                self.0.pop().expect("ensured that there were enough slots").0
            }
        }

        let (mut wasm_module, fn_table_allocated_slots) =
            lir_to_wasm::lir_to_wasm(lir, &mut A(&mut self.free_slots));

        let module_bytes = wasm_module.emit_wasm();

        // SAFETY: it is preconditioned that instantiating the module does not
        // cause undefined behavior in the current instance
        let success = unsafe { instantiate_module(module_bytes.as_ptr(), module_bytes.len()) };
        if !success {
            return (Err(InstallLirError::RuntimeError), module_bytes);
        }

        // return the function pointers to the installed functions.
        let entrypoints = HashMap::from_iter(
            lir.user_functions
                .iter()
                .filter_map(
                    |(lir_fn_id, function)| {
                        if function.is_entrypoint { Some(lir_fn_id) } else { None }
                    },
                )
                .map(|lir_fn_id| {
                    let slot = fn_table_allocated_slots[&lir_fn_id];
                    // TODO for sanity, verify that each entrypoint has the signature
                    // specified in `JitEntrypoint`. However, Wasm should already catch
                    // if we indirectly call a function with the wrong signature.

                    // SAFETY: in the wasm32 target, a function pointer is
                    // represented by a i32 indicating the slot in the
                    // function table, so they literally have the same ABI
                    let fn_ptr = unsafe {
                        std::mem::transmute::<
                            usize,
                            unsafe extern "C" fn(&mut ExecutionContext, *mut PackedAny, u32),
                        >(slot)
                    };
                    (lir_fn_id, fn_ptr)
                }),
        );
        (Ok(Obj { entrypoints }), module_bytes)
    }
}

pub struct Obj {
    entrypoints:
        HashMap<lir::FunctionId, unsafe extern "C" fn(&mut ExecutionContext, *mut PackedAny, u32)>,
}

impl InstalledObj for Obj {
    fn entrypoint(&self, fn_id: lir::FunctionId) -> JitEntrypoint<'_> {
        let fn_ptr = self.entrypoints[&fn_id];
        // SAFETY: according to the safety requirements of the
        // [`InstallLir::install_lir`] the entrypoint functions satisfy the
        // safety requirements of `JitEntrypoint::new`, and this is an
        // entrypoint function.
        unsafe { JitEntrypoint::new(fn_ptr) }
    }
}
