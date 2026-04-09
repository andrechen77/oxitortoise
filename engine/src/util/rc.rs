use std::{
    alloc::{Layout, alloc_zeroed, dealloc},
    cell::Cell,
    mem,
    ptr::{NonNull, drop_in_place},
};

use macro_reflect::{ReflectComponents, reflect};

use crate::util::reflection::Reflect;

#[repr(transparent)]
#[derive(ReflectComponents)]
pub struct ErasedRc {
    /// Points to some unknown value. Before the location pointed to by this
    /// pointer is the metadata prefix.
    ptr_to_value: NonNull<u8>,
}

/// The prefix of an [`ErasedRc`] allocation.
struct ErasedRcPrefix {
    ref_count: Cell<usize>,
    whole_layout: Layout,
    drop_fn: fn(NonNull<u8>),
}

#[reflect]
impl Reflect for ErasedRc {}

impl ErasedRc {
    /// Allocates a new [`ErasedRc`] with the given layout. The memory is
    /// initialized to zero. It is up to the caller to initialize a value at
    /// the memory. The drop function will be called when the value is dropped.
    pub fn new(value_layout: Layout, drop_fn: fn(NonNull<u8>)) -> Self {
        // align the value layout so that the prefix can be placed to its left
        let value_layout = value_layout
            .align_to(mem::align_of::<ErasedRcPrefix>())
            .expect("alignment should be a nonzero power of 2 and the size should not overflow");

        // extend the value layout to fit the prefix. use the extend_packed
        // method because we do not want any padding between the prefix and the
        // value so that ptr - size_of::<ErasedRcPrefix>() points to the prefix.
        // We have already ensured that the value will still be aligned with the
        // align_to operation above.
        // let whole_layout = Layout::new::<ErasedRcPrefix>()
        //     .extend_packed(value_layout)
        //     .expect("if the layout overflows we have bigger problems");

        // since extend_packed is not yet stabilized we use the following equivalent code
        // https://github.com/rust-lang/rust/blob/bad24ccbeceb787d2ad62847196315576a0d4fc7/library/core/src/alloc/layout.rs#L544-L550
        let whole_layout = {
            let orig = Layout::new::<ErasedRcPrefix>();
            let new_size = orig.size() + value_layout.size();
            Layout::from_size_align(new_size, orig.align())
        }
        .unwrap();

        assert!(
            whole_layout.size() > 0,
            "layout size should be greater than 0 because it has at least a AnyRcPrefix"
        );
        // SAFETY: we checked that the size is greater than 0
        let ptr_to_whole = NonNull::new(unsafe { alloc_zeroed(whole_layout) })
            .expect("allocation should not fail")
            .cast::<ErasedRcPrefix>();

        // write the prefix to the allocation
        unsafe {
            ptr_to_whole.write(ErasedRcPrefix { ref_count: Cell::new(1), whole_layout, drop_fn })
        };

        // offset the pointer to point to the actual value
        // SAFETY: the offset fits in a usize (and by implication an isize), and
        // the offset is within the same allocation because the whole allocation
        // is at least the size of the prefix
        let ptr_to_value = unsafe { ptr_to_whole.add(1).cast::<u8>() };

        Self { ptr_to_value }
    }

    fn prefix(&self) -> &ErasedRcPrefix {
        // SAFETY: if the ptr_to_prefix funciton is correct, then it points to a
        // valid value
        unsafe { self.ptr_to_prefix().as_ref() }
    }

    fn ptr_to_prefix(&self) -> NonNull<ErasedRcPrefix> {
        // SAFETY: the pointer is still within the bounds of the allocation after
        // subtraction because it was already offset into the allocation by
        // the size of the prefix. the computed offset fits in an isize because
        // the prefix is not that big. the cast is valid because during
        // initialization the prefix was written to this address
        unsafe { self.ptr_to_value.cast::<ErasedRcPrefix>().sub(1) }
    }
}

impl Clone for ErasedRc {
    fn clone(&self) -> Self {
        self.prefix().increment_count();
        Self { ptr_to_value: self.ptr_to_value }
    }
}

impl Drop for ErasedRc {
    fn drop(&mut self) {
        let prefix = self.prefix();
        prefix.decrement_count();
        if prefix.ref_count.get() == 0 {
            (prefix.drop_fn)(self.ptr_to_value);

            // if the drop function panics, this allocation will not be deallocated.
            // I don't think we care about this but a more robust solution would
            // be like what std uses: create a weak pointer whose destructor would
            // still run even in a panic.

            let ptr_to_prefix = self.ptr_to_prefix();
            let whole_layout = prefix.whole_layout;

            // SAFETY: the pointer points to a valid value and its pointee is
            // never used again
            // should do nothing anyway because the prefix has no destructor
            unsafe { drop_in_place(ptr_to_prefix.as_ptr()) };

            // SAFETY: the prefix is the start of an allocation gotten from
            // `alloc_zeroed` (the same allocator). the layout is the same as
            // the one passed in `new` (and specifically, the layout of the
            // whole allocation and not just the value)
            unsafe { dealloc(ptr_to_prefix.as_ptr().cast(), whole_layout) };
        }
    }
}

impl ErasedRcPrefix {
    fn decrement_count(&self) {
        self.ref_count.set(self.ref_count.get() - 1);
    }

    fn increment_count(&self) {
        self.ref_count.set(self.ref_count.get() + 1);
    }
}

pub mod create_erased_rc {
    use super::*;

    use crate::mir::HostFunctionInfo;

    pub const FN_INFO: HostFunctionInfo = HostFunctionInfo {
        debug_name: "create_erased_rc",
        parameter_types: &[u32::TYPE, u32::TYPE, <fn(NonNull<u8>)>::TYPE],
        return_type: ErasedRc::TYPE,
        link_name: "create_erased_rc",
        link_addr: call as *const u8,
    };

    pub fn call(size: u32, align: u32, drop_fn: fn(NonNull<u8>)) -> ErasedRc {
        let size: usize = size.try_into().unwrap();
        let align: usize = align.try_into().unwrap();
        let layout = Layout::from_size_align(size, align).unwrap();
        ErasedRc::new(layout, drop_fn)
    }
}
