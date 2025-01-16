use crate::sim::value::Float;

use super::raw_parts::{InUntypedVal, TypeId, UntypedVal};
use std::{fmt, mem::ManuallyDrop, ops::{Deref, DerefMut, Index, IndexMut}};

/// A statically-sized array of dynamically-typed NetLogo values. The generic
/// parameter `N` determines how many values can be stored in the array.
pub struct DynTypedArr<const N: usize> {
    types: [TypeId; N],
    values: [UntypedVal; N],
}

impl<const N: usize> DynTypedArr<N> {
    /// Create a new `DynTypedVal` with all values being a default float.
    pub fn new() -> Self {
        Self {
            types: [TypeId::FLOAT; N],
            values: std::array::from_fn(|_| UntypedVal { float: ManuallyDrop::new(Float::default()) }),
        }
    }

    pub fn get_raw(&self, idx: usize) -> (TypeId, &UntypedVal) {
        (self.types[idx], &self.values[idx])
    }

    pub fn get_raw_mut(&mut self, idx: usize) -> (&mut TypeId, &mut UntypedVal) {
        (&mut self.types[idx], &mut self.values[idx])
    }

    pub fn get<T: InUntypedVal>(&self, idx: usize) -> Option<&T> {
        // SAFETY: correct type tag is passed
        unsafe { T::from_untyped(self.types[idx], &self.values[idx]).map(|v| v.deref()) }
    }

    pub fn get_mut<T: InUntypedVal>(&mut self, idx: usize) -> Option<&mut T> {
        // SAFETY: correct type tag is passed
        unsafe { T::from_untyped_mut(self.types[idx], &mut self.values[idx]).map(|v| v.deref_mut()) }
    }

    fn get_dyn(&self, idx: usize) -> &ManuallyDrop<dyn InUntypedVal> {
        // TODO: for the purposes of extensionality, this should ideally use the
        // type id to index into a table of functions (probably defined under the
        // InUntypedVal trait) that can obtain a &dyn NetlogoValueExt

        // SAFETY: the TypeId constants in the match arm patterns ensure that
        // the correct union field is read
        unsafe { match self.get_raw(idx) {
            (TypeId::FLOAT, UntypedVal { float }) => float,
            (TypeId::COLOR, UntypedVal { color }) => color,
            (TypeId::BOOLEAN, UntypedVal { boolean }) => boolean,
            (TypeId::STRING, UntypedVal { string }) => string,
            (TypeId::NOBODY, UntypedVal { nobody }) => nobody,
            (TypeId::TURTLE, UntypedVal { turtle }) => turtle,
            (TypeId::PATCH, UntypedVal { patch }) => patch,
            _ => todo!(),
        }}
    }

    fn get_dyn_mut(&mut self, idx: usize) -> &mut ManuallyDrop<dyn InUntypedVal> {
        // TODO: for the purposes of extensionality, this should ideally use the
        // type id to index into a table of functions (probably defined under the
        // InUntypedVal trait) that can obtain a &dyn NetlogoValueExt

        // SAFETY: the TypeId constants in the match arm patterns ensure that
        // the correct union field is read
        unsafe { match self.get_raw_mut(idx) {
            (&mut TypeId::FLOAT, UntypedVal { float }) => float,
            (&mut TypeId::COLOR, UntypedVal { color }) => color,
            (&mut TypeId::BOOLEAN, UntypedVal { boolean }) => boolean,
            (&mut TypeId::STRING, UntypedVal { string }) => string,
            (&mut TypeId::NOBODY, UntypedVal { nobody }) => nobody,
            (&mut TypeId::TURTLE, UntypedVal { turtle }) => turtle,
            (&mut TypeId::PATCH, UntypedVal { patch }) => patch,
            _ => todo!(),
        }}
    }
}

impl<const N: usize> Index<usize> for DynTypedArr<N> {
    type Output = dyn InUntypedVal;

    fn index(&self, idx: usize) -> &Self::Output {
        self.get_dyn(idx).deref()
    }
}

impl<const N: usize> IndexMut<usize> for DynTypedArr<N> {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        self.get_dyn_mut(idx).deref_mut()
    }
}

impl<const N: usize> Default for DynTypedArr<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Drop for DynTypedArr<N> {
    fn drop(&mut self) {
        for i in 0..N {
            // SAFETY: the value is never used or read afterward, as we are
            // inside the drop function for the DynTypedArr
            unsafe { ManuallyDrop::drop(self.get_dyn_mut(i)) };
        }
    }
}

impl<const N: usize> Clone for DynTypedArr<N> {
    fn clone(&self) -> Self {
        let mut other = Self {
            types: [TypeId::UNINIT; N],
            values: std::array::from_fn(|_| UntypedVal { unit: () }),
        };

        for i in 0..N {
            let (type_id, cloned) = self[i].clone_to_untyped();
            other.types[i] = type_id;
            other.values[i] = cloned;
        }

        other
    }
}

impl<const N: usize> fmt::Debug for DynTypedArr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries((0..N).map(|i| &self[i]))
            .finish()
    }
}
