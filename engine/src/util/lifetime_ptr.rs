use std::marker::PhantomData;

pub struct LifetimePtr<'a> {
    ptr: *const u8,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, T> From<&'a T> for LifetimePtr<'a> {
    fn from(value: &'a T) -> Self {
        Self { ptr: value as *const T as *const u8, _phantom: PhantomData }
    }
}

impl<'a> LifetimePtr<'a> {
    /// # Safety
    ///
    /// The pointer must have come from a `&'a T` with special attention paid
    /// that the lifetime `'a` chosen by the caller is valid.
    pub unsafe fn from_ptr(ptr: *const u8) -> LifetimePtr<'a> {
        Self { ptr, _phantom: PhantomData }
    }

    /// # Safety
    ///
    /// The type must be correct.
    pub unsafe fn cast<T>(self) -> &'a T {
        unsafe { &*self.ptr.cast::<T>() }
    }

    /// Applies a function to the pointer without affecting the lifetime.
    pub fn map(self, f: impl FnOnce(*const u8) -> *const u8) -> Self {
        Self { ptr: f(self.ptr), _phantom: PhantomData }
    }
}

pub struct LifetimePtrMut<'a> {
    ptr: *mut u8,
    _phantom: PhantomData<&'a mut ()>,
}

impl<'a, T> From<&'a mut T> for LifetimePtrMut<'a> {
    fn from(value: &'a mut T) -> Self {
        Self { ptr: value as *mut T as *mut u8, _phantom: PhantomData }
    }
}

impl<'a> LifetimePtrMut<'a> {
    /// # Safety
    ///
    /// The pointer must have come from a `&'a mut T` with special attention paid
    /// that the lifetime `'a` chosen by the caller is valid.
    pub unsafe fn from_ptr(ptr: *mut u8) -> LifetimePtrMut<'a> {
        Self { ptr, _phantom: PhantomData }
    }

    /// # Safety
    ///
    /// The type must be correct.
    pub unsafe fn cast<T>(self) -> &'a mut T {
        unsafe { &mut *self.ptr.cast::<T>() }
    }

    /// Applies a function to the pointer without affecting the lifetime.
    pub fn map(self, f: impl FnOnce(*mut u8) -> *mut u8) -> Self {
        Self { ptr: f(self.ptr), _phantom: PhantomData }
    }
}
