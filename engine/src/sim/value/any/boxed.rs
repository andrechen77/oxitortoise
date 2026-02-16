use std::fmt::Debug;
use std::{
    alloc::{self, Layout},
    ptr::NonNull,
};

use crate::sim::value::{NlBool, NlFloat, NlList, NlString};
use crate::util::reflection::{ConcreteTy, Reflect};

pub struct BoxedAny {
    /// Always points to a valid [`ConcreteTy`] in memory, as if this field were
    /// actually `Box<ConcreteTy>`. This `ConcreteTy` acts as a type tag for the
    /// actual value being stored, which follows in memory after `ConcreteTy`,
    /// where the offset is determined by [`std::alloc::Layout::extend`]. The
    /// type itself is immutable, so the `ConcreteTy` can never change.
    inner: NonNull<ConcreteTy>,
}

/// `BoxedAny` points to a pair of a `ConcreteTy` followed by the actual value
/// of type `T`. This function returns the overall layout of the whole pair as
/// well as the offset into the allocation where the value starts.
fn layout_and_val_offset(val_layout: Layout) -> (Layout, usize) {
    Layout::new::<ConcreteTy>().extend(val_layout).unwrap()
}

impl BoxedAny {
    pub fn new<T: Reflect>(value: T) -> Self {
        // allocate memory for the value and its type tag
        let (layout, value_start) = layout_and_val_offset(T::CONCRETE_TY.info().layout);
        assert!(layout.size() > 0, "layout size must be greater than 0");
        // SAFETY: we checked that the size is greater than 0
        let all_ptr =
            NonNull::new(unsafe { alloc::alloc(layout) }).expect("allocation should not fail");

        // move the type tag into the allocated memory
        // SAFETY: the ptr is valid for writes and the allocator should have
        // returned a properly aligned pointer
        unsafe { std::ptr::write(all_ptr.cast::<ConcreteTy>().as_ptr(), T::CONCRETE_TY) };

        // move the value into the allocated memory
        // SAFETY: we trust that the Layout::extend API correctly gave an offset
        // into the allocation
        let val_ptr = unsafe { all_ptr.byte_add(value_start) };
        // SAFETY: the ptr is valid for writes and we trust that the
        // Layout::extend API resulted in an aligned offset
        unsafe { std::ptr::write(val_ptr.cast::<T>().as_ptr(), value) };

        Self { inner: all_ptr.cast() }
    }

    /// Creates a new [`BoxedAny`] from a pointer to a [`ConcreteTy`] followed
    /// by the actual value.
    ///
    /// # Safety
    ///
    /// The pointer must have come from a call to [`BoxedAny::as_raw`].
    pub unsafe fn from_raw(ptr: NonNull<ConcreteTy>) -> Self {
        Self { inner: ptr }
    }

    pub fn as_raw(&self) -> NonNull<ConcreteTy> {
        self.inner
    }

    fn ty(&self) -> ConcreteTy {
        // SAFETY: this pointer is always valid when used to access `ConcreteTy`
        // per this type's invariants
        unsafe { *self.inner.as_ptr() }
    }

    /// Obtains a pointer to the actual value stored in this [`BoxedAny`].
    /// Panics if the attempted access type does not match the type tag.
    fn ptr_to_val<T: Reflect>(&self) -> NonNull<T> {
        let ty = T::CONCRETE_TY;
        assert!(ty == self.ty(), "type mismatch");
        let (_, offset) = layout_and_val_offset(ty.info().layout);
        // SAFETY: since the type tag passed the assertion, we know that the
        // type must be correct and therefore the pointer derived from assuming
        // that type must be in bounds
        unsafe { self.inner.byte_add(offset) }.cast()
    }

    /// If the specified layout corresponds to the actul type of the value
    /// stored in this [`BoxedAny`], then returns a pointer to that value.
    ///
    /// # Safety
    ///
    /// The layout must be correct for the actual type of the value stored in
    /// this [`BoxedAny`].
    unsafe fn ptr_to_val_with_layout(&self, layout: Layout) -> NonNull<u8> {
        let (_, offset) = layout_and_val_offset(layout);
        // SAFETY: it is preconditioned that the layout is correct and therefore
        // the pointer derived from assuming that type must be in bounds
        unsafe { self.inner.byte_add(offset) }.cast()
    }

    pub fn deref_as<T: Reflect>(&self) -> &T {
        let ptr = self.ptr_to_val::<T>();
        // SAFETY: the pointer is valid and the type matches
        unsafe { &*ptr.as_ptr() }
    }

    pub fn deref_as_mut<T: Reflect>(&mut self) -> &mut T {
        let ptr = self.ptr_to_val::<T>();
        // SAFETY: the pointer is valid and the type matches
        unsafe { &mut *ptr.as_ptr() }
    }
}

impl Drop for BoxedAny {
    fn drop(&mut self) {
        let type_info = self.ty().info();
        // SAFETY: the layout is correct because it came from the actual type
        // tag
        let val_ptr = unsafe { self.ptr_to_val_with_layout(type_info.layout) };
        if let Some(drop_fn) = type_info.drop_fn {
            // SAFETY: the passed pointer is valid per `BoxedAny`'s invariants, and
            // since we are in the drop implementation, it will never be used again
            unsafe { drop_fn(val_ptr.as_ptr()) };
        }

        // now that the value has been dropped, we can/should dealloc the memory
        let (all_layout, _) = layout_and_val_offset(type_info.layout);
        unsafe { alloc::dealloc(self.inner.as_ptr().cast(), all_layout) };
    }
}

impl Debug for BoxedAny {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.ty() == NlBool::CONCRETE_TY {
            write!(f, "{:?}", self.deref_as::<NlBool>())?;
        } else if self.ty() == NlFloat::CONCRETE_TY {
            write!(f, "{:?}", self.deref_as::<NlFloat>())?;
        } else if self.ty() == NlList::CONCRETE_TY {
            write!(f, "{:?}", self.deref_as::<NlList>())?;
        } else if self.ty() == NlString::CONCRETE_TY {
            write!(f, "{:?}", self.deref_as::<NlString>())?;
        } else {
            write!(f, "BoxedAny(unknown type {:?})", self.ty())?;
        }

        Ok(())
    }
}
