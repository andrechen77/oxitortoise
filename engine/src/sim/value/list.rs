use std::ops::{Index, IndexMut};

use crate::{
    mir::reflection::{Reflect, Type, TypeInfo},
    sim::value::{NlFloat, PackedAny},
};

#[derive(Default, Debug)]
pub struct NlList(Vec<PackedAny>);

impl NlList {
    pub fn new() -> Self {
        Self::default()
    }
}

unsafe impl Reflect for NlList {
    const TYPE: Type = Type::new(&TypeInfo::new_opaque::<NlList>("NlList"));
}

impl NlList {
    pub fn push(&mut self, element: PackedAny) {
        self.0.push(element);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn swap_remove(&mut self, index: usize) -> PackedAny {
        self.0.swap_remove(index)
    }
}

impl Index<NlFloat> for NlList {
    type Output = PackedAny;

    fn index(&self, index: NlFloat) -> &Self::Output {
        // TODO verify that float->usize conversion matches NetLogo behavior
        &self.0[index.get() as usize]
    }
}

impl IndexMut<NlFloat> for NlList {
    fn index_mut(&mut self, index: NlFloat) -> &mut Self::Output {
        // TODO verify that float->usize conversion matches NetLogo behavior
        &mut self.0[index.get() as usize]
    }
}

impl Index<usize> for NlList {
    type Output = PackedAny;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for NlList {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}
