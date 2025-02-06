#[cfg(not(feature = "runtime-unsafety"))]
pub type RefCell<T> = std::cell::RefCell<T>;

#[cfg(feature = "runtime-unsafety")]
pub type RefCell<T> = unsafe_ref_cell::UnsafeRefCell<T>;

#[cfg(feature = "runtime-unsafety")]
mod unsafe_ref_cell {
    use std::cell::UnsafeCell;

    #[derive(Debug, Default)]
    #[repr(transparent)]
    pub struct UnsafeRefCell<T: ?Sized> {
        value: UnsafeCell<T>,
    }

    pub struct UnsafeRef<'a, T: ?Sized + 'a> {
        value: &'a T,
    }

    impl<T: ?Sized> std::ops::Deref for UnsafeRef<'_, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.value
        }
    }

    pub struct UnsafeRefMut<'a, T: ?Sized + 'a> {
        value: &'a mut T,
    }

    impl<T: ?Sized> std::ops::Deref for UnsafeRefMut<'_, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.value
        }
    }

    impl<T: ?Sized> std::ops::DerefMut for UnsafeRefMut<'_, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.value
        }
    }

    impl<T> UnsafeRefCell<T> {
        pub fn new(value: T) -> Self {
            Self {
                value: UnsafeCell::new(value),
            }
        }
    }

    impl<T: ?Sized> UnsafeRefCell<T> {
        pub fn borrow(&self) -> UnsafeRef<'_, T> {
            // SAFETY: The caller must ensure that no other references are alive
            // when calling this function. This is actually an unsafe function
            // and correct usage should be checked by substituting the safe
            // version of RefCell
            UnsafeRef {
                value: unsafe { &*self.value.get() },
            }
        }

        pub fn borrow_mut(&self) -> UnsafeRefMut<'_, T> {
            // SAFETY: The caller must ensure that no other references are alive
            // when calling this function. This is actually an unsafe function
            // and correct usage should be checked by substituting the safe
            // version of RefCell
            UnsafeRefMut {
                value: unsafe { &mut *self.value.get() },
            }
        }

        pub fn get_mut(&mut self) -> &mut T {
            // SAFETY: It is statically ensured that no other references are
            // alive when calling this function.
            unsafe { &mut *self.value.get() }
        }
    }
}
