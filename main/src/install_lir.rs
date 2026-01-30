use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    sync::{LazyLock, Mutex},
};

use engine::{
    exec::{
        ExecutionContext,
        jit::{InstallLirError, JitEntrypoint},
    },
    lir,
};

static FUNCTION_INSTALLER: LazyLock<Mutex<FunctionInstaller>> = LazyLock::new(|| {
    // SAFETY: the function installer is the only code that interacts with the
    // instantiation APIs. Since this is called in a LazyLock, it is only
    // instantiated once. Therefore, it is safe to assume that no other code is
    // interacting with the instantiation APIs.
    Mutex::new(unsafe { FunctionInstaller::new() })
});

pub unsafe fn install_lir(
    lir: &lir::Program,
) -> Result<(HashMap<lir::FunctionId, JitEntrypoint>, Vec<u8>), (InstallLirError, Vec<u8>)> {
    unsafe {
        FUNCTION_INSTALLER
            .lock()
            .map_err(|_| (InstallLirError::InstallerPoisoned, vec![]))?
            .install_lir(lir)
    }
}

#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    /// Instantiates the module at the specified bytes (creating an "auxiliary
    /// instance"). All exports of the currently running instance will be
    /// imported by the auxiliary instance under the "main_module" namespace.
    /// This includes:
    /// - memory "memory": the memory used by the main module. The auxiliary
    /// instance will be able to read and write our memory.
    /// - table "__indirect_function_table": the function table used by the main
    /// module. The auxiliary instance will be able to install its own functions
    /// into this function table, which we will be able to call.
    /// - global "__stack_pointer": the stack pointer used by the main module.
    /// The auxiliary instance will use this stack to store its own local
    /// variables.
    /// - all function exports. The auxiliary instance will be able to call
    /// our functions.
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
    ) -> Result<(HashMap<lir::FunctionId, JitEntrypoint>, Vec<u8>), (InstallLirError, Vec<u8>)>
    {
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

        // TODO for sanity, verify that each entrypoint has the signature
        // specified in `JitEntrypoint`. However, Wasm should already catch
        // if we indirectly call a function with the wrong signature.

        let (mut wasm_module, fn_table_allocated_slots) =
            lir_to_wasm::lir_to_wasm(lir, &mut A(&mut self.free_slots));

        let module_bytes = wasm_module.emit_wasm();

        // SAFETY: it is preconditioned that instantiating the module does not
        // cause undefined behavior in the current instance
        let success = unsafe { instantiate_module(module_bytes.as_ptr(), module_bytes.len()) };
        if !success {
            return Err((InstallLirError::RuntimeError, module_bytes));
        }

        // return the function pointers to the installed functions.
        // TODO(wishlist) we might be able to transmute here
        let jit_entries =
            HashMap::from_iter(fn_table_allocated_slots.into_iter().map(|(lir_fn_id, slot)| {
                (
                    lir_fn_id,
                    JitEntrypoint::new(
                        // SAFETY: in the wasm32 target, a function pointer is
                        // represented by a i32 indicating the slot in the
                        // function table, so they literally have the same ABI
                        unsafe {
                            std::mem::transmute::<
                                usize,
                                extern "C" fn(&mut ExecutionContext, *mut u8),
                            >(slot)
                        },
                    ),
                )
            }));
        Ok((jit_entries, module_bytes))
    }
}
