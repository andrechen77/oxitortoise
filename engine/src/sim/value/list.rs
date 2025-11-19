use std::ops::{Index, IndexMut};

use crate::{
    sim::value::{DynBox, NlBox, NlFloat, r#box::generate_box_type_info},
    util::reflection::{ConcreteTy, Reflect, TypeInfo},
};

pub struct NlList(Vec<DynBox>);

static LIST_TYPE_INFO: TypeInfo = generate_box_type_info::<NlList>("List");
impl Reflect for NlBox<NlList> {
    const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&LIST_TYPE_INFO);
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
