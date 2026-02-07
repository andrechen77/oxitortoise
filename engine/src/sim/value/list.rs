use std::ops::{Index, IndexMut};

use crate::{
    sim::value::{DynBox, NlBox, NlFloat, r#box::generate_box_type_info},
    util::reflection::{ConcreteTy, Reflect, TypeInfo},
};

#[derive(Default)]
pub struct NlList(Vec<DynBox>);

impl NlList {
    pub fn new() -> Self {
        Self::default()
    }
}

static LIST_TYPE_INFO: TypeInfo = generate_box_type_info::<NlList>("List");
impl Reflect for NlBox<NlList> {
    const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&LIST_TYPE_INFO);
}

impl NlList {
    pub fn push(&mut self, element: DynBox) {
        self.0.push(element);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn swap_remove(&mut self, index: usize) -> DynBox {
        self.0.swap_remove(index)
    }
}

impl Index<NlFloat> for NlList {
    type Output = DynBox;

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
    type Output = DynBox;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for NlList {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}
