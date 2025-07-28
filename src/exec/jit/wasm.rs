use std::collections::BinaryHeap;

use super::JitFn;
use walrus::{ir::Value, ConstExpr, ElementItems, ElementKind, FunctionId, Module, TableId};

unsafe extern "C" {
    // TODO talk about the limits of the memory and function table. should it
    // have no max?
    /// Instantiates the module at the specified bytes (creating an "auxiliary
    /// instance"). The auxiliary instance will be provided imports from the
    /// "main" namespace: a memory "memory" and a function table "table", which
    /// be shared with the current instance. Returns whether it was successful.
    ///
    /// # Safety
    ///
    /// Since the auxiliary instance will be executed with the current
    /// instance's memory and function table, so must not cause any undefined
    /// behavior.
    fn instantiate_module(module_bytes_start: *const u8, module_bytes_len: usize) -> bool;

    // TODO this shouldn't have to be imported from the host environment. it
    // would be ideal if we could do this by emitting Wasm instructions in the
    // current module directly to just grow the table.
    /// Creates new slots in the function table by growing the table, and
    /// returns the index of the first new slot in the table. Returns `u32::MAX`
    /// if the operation could not be completed.
    safe fn grow_function_table(num_slots: u32) -> u32;
}

/// Tracks state required to install additional functions to the current
/// instance's function table.
pub struct FunctionInstaller {
    free_slots: BinaryHeap<u32>,
}

impl FunctionInstaller {
    /// # Safety
    ///
    /// A `FunctionInstaller` assumes that it is the only one interacting with
    /// the current instance's function table and the imported instantiation
    /// APIs. Therefore, a FunctionInstaller cannot exist at any time other code
    /// might be attempting to make those interactions, including when other
    /// `FunctionInstaller`s exist.
    pub unsafe fn new() -> Self {
        Self {
            free_slots: BinaryHeap::new(),
        }
    }

    /// Installs the specified functions of the specified module into the
    /// current instance's function table, and returns function pointers to
    /// those functions in the same order.
    ///
    /// # Safety
    ///
    /// The destination table must be an Id that refers to a table imported from
    /// the current instance, in order for the resulting functions to be
    /// visible. In addition, the auxiliary module must not cause undefined
    /// behavior when instantiated: this means the start function, if it exists,
    /// must not corrupt memory, and it must not modify the function table of
    /// the current module (though it is allowed to modify function tables it
    /// declares locally). The functions being installed must have the right
    /// type signature (see [`JitFn`])
    pub unsafe fn install_functions(
        &mut self,
        mut module: Module,
        functions_to_install: &[FunctionId],
        destination_table: TableId,
    ) -> Result<Vec<JitFn>, ()> {
        // find slots in the function table to put these functions
        let num_required_slots = functions_to_install.len();
        if self.free_slots.len() < num_required_slots {
            // acquire enough additional slots
            let num_new_slots = num_required_slots - self.free_slots.len();
            let first_new_slot = grow_function_table(num_new_slots as u32);
            for new_slot in first_new_slot..(first_new_slot + num_new_slots as u32) {
                self.free_slots.push(new_slot);
            }
        }

        // install the functions into the current instance's function table
        let mut slots = Vec::with_capacity(num_required_slots);
        for &function_id in functions_to_install {
            let slot = self
                .free_slots
                .pop()
                .expect("ensured that there were enough slots");
            slots.push(slot);
            module.elements.add(
                ElementKind::Active {
                    table: destination_table,
                    offset: ConstExpr::Value(Value::I32(slot as i32)),
                },
                ElementItems::Functions(vec![function_id]),
            );
        }

        // instantiate the module
        let module_bytes = module.emit_wasm();
        // SAFETY: it is preconditioned that instantiating the module does not
        // cause undefined behavior in the current instance
        let success = unsafe { instantiate_module(module_bytes.as_ptr(), module_bytes.len()) };
        if !success {
            return Err(());
        }

        // return the function pointers to the installed functions

        // SAFETY: in the wasm32 target, a function pointer is represented by a
        // i32 indicating the slot in the function table, so they literally have
        // the same ABI
        Ok(unsafe { std::mem::transmute::<Vec<u32>, Vec<JitFn>>(slots) })
    }
}
