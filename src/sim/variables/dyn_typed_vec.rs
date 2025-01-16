use std::ops::{Index, IndexMut};

use super::{dyn_typed_arr::DynTypedArr, raw_parts::InUntypedVal};

/// A growable array of dynamically-typed NetLogo values. The generic parameter
/// `N` determines how many values are stored directly in the struct, with all
/// other values stored behind a pointer.
#[derive(Debug, Clone)]
pub struct DynTypedVec<const N: usize> {
    direct_values: DynTypedArr<N>,
    // TODO add len and capacity fields, if necessary
    // TODO add spilled values
}

impl<const N: usize> DynTypedVec<N> {
    pub fn new() -> Self {
        Self {
            direct_values: DynTypedArr::new(),
        }
    }

    pub fn reset(&mut self) {
        self.direct_values = Default::default();
    }

    pub fn get<T: InUntypedVal>(&self, idx: usize) -> Option<&T> {
        self.direct_values.get(idx)
    }

    pub fn get_mut<T: InUntypedVal>(&mut self, idx: usize) -> Option<&mut T> {
        self.direct_values.get_mut(idx)
    }
}

impl<const N: usize> Default for DynTypedVec<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Index<usize> for DynTypedVec<N> {
    type Output = dyn InUntypedVal;

    fn index(&self, index: usize) -> &Self::Output {
        &self.direct_values[index]
    }
}

impl<const N: usize> IndexMut<usize> for DynTypedVec<N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.direct_values[index]
    }
}

