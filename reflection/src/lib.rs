use std::sync::LazyLock;

pub use dyn_ptr::{DynPtr, DynPtrMut, HasDynPtr};
pub use dyn_type::{CreateDynType, DynType, DynTypeArray, DynTypeStruct};
pub use lifetime_ptr::{LifetimePtr, LifetimePtrMut};
pub use static_type::{CloneKind, StaticType, StaticTypeInfo};

mod dyn_ptr;
mod dyn_type;
mod lifetime_ptr;
pub mod mir;
mod static_type;

/// A trait to indicate that the compiler can generate code to manipulate values
/// of this type.
///
/// # Safety
///
/// Implementors must guarantee that the associated `Type` is correct, as
/// the information will be used to generate and run unsafe code.
pub unsafe trait Reflect {
    const STATIC_TYPE: StaticType;

    const DYN_TYPE: &LazyLock<DynType>;

    fn dyn_type() -> DynType {
        (*Self::DYN_TYPE).clone()
    }
}
