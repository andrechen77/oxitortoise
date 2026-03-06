use std::alloc::Layout;

use either::Either;

use crate::{
    mir::{LocalId, Place, Projection, builder::FunctionBuilder},
    util::lifetime_ptr::LifetimePtrMut,
};

// TODO what to do about lifetimes? could cause unsafety and sadness

/// A trait to indicate that the compiler can generate code to manipulate values
/// of this type.
///
/// # Safety
///
/// Implementors must guarantee that the associated `Type` is correct, as
/// the information will be used to generate and run unsafe code.
pub unsafe trait Reflect {
    const TYPE: Type;
}

/// Information about a type that is used by the engine to generate code that
/// manipulates values of the corresponding type.
#[derive(Debug, Clone)]
pub struct TypeInfo {
    /// An identifier that is always different for types that differ by more
    /// than just lifetimes. This is the only fields used to check for type
    /// identity.
    ///
    /// Between two types that differ only in lifetimes, this will be the same.
    /// For example, `&mut T<'_>` (for type `T` known at compile time) will use
    /// the type id of `&'static mut T<'static>`, even though they are not the
    /// same type.
    ///
    /// Types known at compile time are given a [`std::any::TypeId`], while
    /// types registered at runtime are given a unique integer.
    pub unique_id: Either<std::any::TypeId, u32>,
    pub debug_name: &'static str,
    pub layout: Option<Layout>,
    /// Whether this type is valid at the all-zero bit pattern *and* represents
    /// the numeric value 0.0.
    pub is_zeroable: bool,
    /// The drop function for this type. As is standard for drop functions, this
    /// should deallocate any memory that the value itself owns, but does not
    /// deallocate the memory that the value itself inhabits (that is the
    /// responsibility of whoever owns the value, i.e. the caller of this
    /// function). None indicates that the type is `Copy`.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the passed pointer is a valid pointer to
    /// T that can be dropped, and that that value will never be used again.
    pub drop_fn: Option<unsafe fn(*mut u8)>,
    /// Information about the memory representation of this type.
    pub mem_desc: &'static MemDesc,
}

unsafe fn drop_impl<T>(ptr: *mut u8) {
    unsafe {
        std::ptr::drop_in_place(ptr as *mut T);
    }
}

impl TypeInfo {
    pub const fn new_drop<T: 'static>(
        debug_name: &'static str,
        mem_desc: &'static MemDesc,
    ) -> Self {
        Self {
            unique_id: Either::Left(std::any::TypeId::of::<T>()),
            debug_name,
            layout: Some(Layout::new::<T>()),
            is_zeroable: false,
            drop_fn: Some(drop_impl::<T>),
            mem_desc,
        }
    }

    pub const fn new_drop_zeroable<T: 'static>(
        debug_name: &'static str,
        mem_desc: &'static MemDesc,
    ) -> Self {
        Self {
            unique_id: Either::Left(std::any::TypeId::of::<T>()),
            debug_name,
            layout: Some(Layout::new::<T>()),
            is_zeroable: true,
            drop_fn: Some(drop_impl::<T>),
            mem_desc,
        }
    }

    pub const fn new_copy<T: Copy + 'static>(
        debug_name: &'static str,
        is_zeroable: bool,
        mem_desc: &'static MemDesc,
    ) -> Self {
        Self {
            unique_id: Either::Left(std::any::TypeId::of::<T>()),
            debug_name,
            layout: Some(Layout::new::<T>()),
            is_zeroable,
            drop_fn: None,
            mem_desc,
        }
    }

    // pub const fn new_mut_ref_to<T: Reflect + 'static>(debug_name: &'static str) -> Self {
    //     Self {
    //         unique_id: Either::Left(std::any::TypeId::of::<&'static mut T>()),
    //         debug_name,
    //         layout: Some(Layout::new::<&mut T>()),
    //         is_zeroable: false,
    //         drop_fn: None, // mut refs have no destructor, so this is correct
    //         mem_desc: MemDesc::IsPointerTo(Box::new(T::TYPE.info().mem_desc)),
    //     }
    // }

    // types that can only be referenced through pointer
    pub const fn new_opaque<T: 'static>(debug_name: &'static str) -> Self {
        Self {
            unique_id: Either::Left(std::any::TypeId::of::<T>()),
            debug_name,
            layout: Some(Layout::new::<T>()),
            is_zeroable: false,
            drop_fn: Some(drop_impl::<T>),
            mem_desc: &MemDesc::None,
        }
    }

    // pub fn concrete_ty(&'static self) -> ConcreteTy {
    //     ConcreteTy::Static(self)
    // }
}

/// A concrete type representation in the NetLogo engine. The same NetLogo
/// language type may have multiple concrete type representations.
#[derive(Clone, Copy, derive_more::Debug)]
#[debug("{}", self.info().debug_name)]
pub struct Type(&'static TypeInfo);

impl Type {
    pub const fn new(info: &'static TypeInfo) -> Self {
        Type(info)
    }

    pub const fn info(&self) -> &'static TypeInfo {
        &self.0
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        self.info().unique_id == other.info().unique_id
    }
}

// impl From<&Type> for Type {
//     fn from(ty: &Type) -> Self {
//         ty.clone()
//     }
// }

// impl From<&'static TypeInfo> for Type {
//     fn from(info: &'static TypeInfo) -> Self {
//         Type(info)
//     }
// }

// impl From<&&'static TypeInfo> for ConcreteTy {
//     fn from(info: &&'static TypeInfo) -> Self {
//         ConcreteTy::Static(info)
//     }
// }

unsafe impl Reflect for () {
    const TYPE: Type = Type::new(&TypeInfo::new_copy::<()>("()", true, &MemDesc::None));
}

unsafe impl Reflect for u32 {
    const TYPE: Type = Type::new(&TypeInfo::new_copy::<u32>(
        "u32",
        true,
        &MemDesc::IsPrimitive(lir::ValType::I32),
    ));
}

unsafe impl Reflect for f64 {
    const TYPE: Type = Type::new(&TypeInfo::new_copy::<f64>(
        "f64",
        true,
        &MemDesc::IsPrimitive(lir::ValType::F64),
    ));
}

/// Represents a description of how a type is stored in memory and how it can
/// be accessed.
#[derive(Debug, Clone)]
pub enum MemDesc {
    /// Asserts that the value is a pointer to a type that satisfies the given assertion.
    IsPointerTo(Box<MemDesc>),
    /// Same as [`MemDesc::IsPointerTo`], but the pointee is known at compile
    /// time.
    ///
    /// Allows for MemDesc to be const promoted.
    StaticIsPointerTo { pointee: &'static MemDesc },
    /// Asserts that the value has fields at the specified byte offsets which
    /// each satisfy their respective assertions.
    HasFields { fields: Vec<(usize, MemDesc)> },
    /// Same as [`MemDesc::HasFields`], but the fields are known at compile
    /// time.
    ///
    /// Allows for MemDesc to be const promoted.
    StaticHasFields { fields: &'static [(usize, MemDesc)] },
    /// Asserts that the value is a array having an element type that satisfies
    /// the given assertion. This does not check the length of the array.
    IsArrayOf { element: Box<MemDesc> },
    /// Asserts that the value is a specific concrete type.
    IsType(Type),
    /// Asserts that the value is a primitive type.
    IsPrimitive(lir::ValType),
    /// No assertion.
    None,
}

impl MemDesc {
    pub fn project(&self, projection: Projection) -> &Self {
        match (self, projection) {
            (MemDesc::IsPointerTo(pointee), Projection::Deref) => pointee.as_ref(),
            (MemDesc::HasFields { fields }, Projection::Field { byte_offset }) => {
                let Some((_, field)) = fields.iter().find(|(offset, _)| *offset == byte_offset)
                else {
                    panic!("Field at byte offset {} not found", byte_offset);
                };
                field
            }
            (MemDesc::IsArrayOf { element }, Projection::Index(_index)) => element,
            (desc, projection) => {
                panic!(
                    "Cannot project memory descriptor {:?} with projection: {:?}",
                    desc, projection
                )
            }
        }
    }

    /// Returns a mutable reference to the memory descriptor of the value at the
    /// specified projection from the original type. If the required projection
    /// is not possible on this memory descriptor but can be compatibly added,
    /// this function will add the required projection. If the required
    /// projection is incompatible with this memory descriptor, this function will
    /// panic.
    pub fn project_mut_with_modify(&mut self, projection: &Projection) -> &mut Self {
        match projection {
            Projection::Deref => match self {
                MemDesc::IsPointerTo(pointee) => pointee,
                MemDesc::None => {
                    *self = MemDesc::IsPointerTo(Box::new(MemDesc::None));
                    let MemDesc::IsPointerTo(pointee) = self else {
                        unreachable!("we just inserted it");
                    };
                    pointee.as_mut()
                }
                other => panic!(
                    "cannot project memory descriptor {:?} with projection: {:?}",
                    other, projection
                ),
            },
            Projection::Field { byte_offset } => match self {
                MemDesc::HasFields { fields } => {
                    // find the field at the given byte offset, or insert a new one
                    // with an empty assertion if it doesn't exist
                    let i = match fields.binary_search_by_key(byte_offset, |(offset, _)| *offset) {
                        Ok(i) => i,
                        Err(i) => {
                            fields.insert(i, (*byte_offset, MemDesc::None));
                            i
                        }
                    };
                    &mut fields[i].1
                }
                MemDesc::None => {
                    *self = MemDesc::HasFields { fields: vec![(*byte_offset, MemDesc::None)] };
                    let MemDesc::HasFields { fields } = self else {
                        unreachable!("we just inserted it");
                    };
                    &mut fields[0].1
                }
                other => panic!(
                    "cannot project memory descriptor {:?} with projection: {:?}",
                    other, projection
                ),
            },
            Projection::Index(_index) => match self {
                MemDesc::IsArrayOf { element } => element.as_mut(),
                MemDesc::None => {
                    *self = MemDesc::IsArrayOf { element: Box::new(MemDesc::None) };
                    let MemDesc::IsArrayOf { element } = self else {
                        unreachable!("we just inserted it");
                    };
                    element.as_mut()
                }
                other => panic!(
                    "cannot project memory descriptor {:?} with projection: {:?}",
                    other, projection
                ),
            },
        }
    }

    pub fn modify_type(&mut self, ty: Type) {
        match self {
            MemDesc::IsType(asserted_ty) => {
                assert_eq!(*asserted_ty, ty);
            }
            MemDesc::None => {
                *self = MemDesc::IsType(ty);
            }
            _ => panic!("A type descriptor {:?} was asserted to be {:?}", self, ty),
        }
    }

    /// Asserts that the value is a specific concrete type, and panics if it is
    /// not.
    pub fn assert_type(&self, ty: Type) {
        match self {
            MemDesc::IsType(asserted_ty) => {
                assert_eq!(*asserted_ty, ty);
            }
            _ => panic!("A memory descriptor {:?} was asserted to be {:?}", self, ty),
        }
    }
}

// TODO move elsewhere
pub struct PlaceWithMemDesc<'a> {
    place: Place,
    mem_desc: &'a mut MemDesc,
}

impl<'a> PlaceWithMemDesc<'a> {
    pub fn new(place: Place, mem_desc: &'a mut MemDesc) -> Self {
        Self { place, mem_desc }
    }

    pub fn place(&self) -> &Place {
        &self.place
    }

    pub fn ty(&self) -> &MemDesc {
        self.mem_desc
    }

    pub fn into_place(self) -> Place {
        self.place
    }

    pub fn proj_deref(self) -> Self {
        let projection = Projection::Deref;
        Self {
            mem_desc: self.mem_desc.project_mut_with_modify(&projection),
            place: self.place.proj(projection),
        }
    }

    pub fn proj_field(self, byte_offset: usize) -> Self {
        let projection = Projection::Field { byte_offset };
        Self {
            mem_desc: self.mem_desc.project_mut_with_modify(&projection),
            place: self.place.proj(projection),
        }
    }

    pub fn proj_index(self, index: LocalId) -> Self {
        let projection = Projection::Index(index);
        Self {
            mem_desc: self.mem_desc.project_mut_with_modify(&projection),
            place: self.place.proj(projection),
        }
    }
}

/// Indicates that the type contains a pointer that points to dynamically typed
/// data.
pub unsafe trait DynPtr {
    /// Builds the MIR code to get the data pointer of the dynamically typed
    /// data, where `self_place` contains a value of `Self`.
    fn write_mir_get_data_ptr<'a>(
        builder: &'a mut FunctionBuilder,
        self_place: Place,
    ) -> PlaceWithMemDesc<'a>;

    /// Returns a pointer to the dynamically typed data as well as a memory
    /// descriptor. Note that the memory descriptor is for the value being pointed
    /// to, not the pointer itself (i.e. it is not always
    /// [`MemDesc::IsPointerTo`]).
    fn data_ptr_mut(&mut self) -> (LifetimePtrMut<'_>, MemDesc);
}
