use std::{alloc::Layout, fmt, ptr::NonNull, sync::Arc};

use tracing::trace;

use crate::{
    mir::{Place, Projection, builder::FunctionBuilder},
    util::{
        lifetime_ptr::{LifetimePtr, LifetimePtrMut},
        reflection::{CloneKind, Reflect, Type},
    },
};

/// A trait to indicate how accesses into values of this type can be generated.
///
/// # Safety
///
/// Implementors must guarantee that the associated `mir_type` is correct, as
/// the information will be used to generate and run unsafe code.
pub unsafe trait MirReflect {
    fn mir_type() -> MirType;
}

#[derive(Clone, Default, PartialEq, Eq)]
pub enum MirType {
    /// A non-owning, mutable reference to the specified type. Equivalent to
    /// `&mut T` but doesn't have a uniqueness guarantee.
    Ref(Box<MirType>),
    /// An aggregate type with the specified fields.
    Struct(Arc<MirTypeStruct>),
    /// An array having an element type that satisfies the given assertion. If
    /// the length is specified, then the array has exactly that many elements,
    /// otherwise it has a statically unknown length.
    Array(Arc<MirTypeArray>),
    // /// A primitive type.
    // Primitive(lir::ValType),
    /// A statically known type
    StaticStruct(Arc<MirTypeStaticStruct>),
    /// We know nothing about this type.
    #[default]
    None,
}

#[derive(PartialEq, Eq)]
pub struct MirTypeStruct {
    /// The fields of the type. This may not be a complete list.
    pub fields: Vec<(usize, MirType)>,
    pub overall: Layout,
}

#[derive(PartialEq, Eq)]
pub struct MirTypeArray {
    pub element: MirType,
    pub length: Option<usize>,
}

#[derive(PartialEq, Eq)]
pub struct MirTypeStaticStruct {
    pub static_ty: Type,
    /// Any additional information about the fields of this struct.
    pub fields: Vec<(usize, MirType)>,
}

impl MirType {
    pub fn layout(&self) -> Layout {
        match self {
            MirType::Ref(_) => Layout::new::<*const u8>(),
            MirType::Struct(struct_def) => struct_def.overall,
            MirType::Array(_) => {
                unimplemented!(
                    "would use the Layout::repeat function on the element layout to get the layout of the whole array"
                )
            }
            // MirType::Primitive(_ty) => todo!("TODO get the layout of a primitive type"),
            MirType::StaticStruct(struct_def) => struct_def.static_ty.layout.unwrap(),
            MirType::None => panic!("Cannot get layout"),
        }
    }

    pub fn static_ty(&self) -> Option<Type> {
        trace!("getting static_ty of: {:?}", self);
        match self {
            MirType::StaticStruct(struct_def) => Some(struct_def.static_ty),
            _ => None,
        }
    }

    pub fn has_drop_fn(&self) -> bool {
        match self {
            MirType::StaticStruct(struct_def) => struct_def.static_ty.drop_fn.is_some(),
            MirType::Struct(_struct_def) => {
                unimplemented!("We shouldn't be dropping custom structs...")
            }
            _ => false,
        }
    }

    pub fn clone_kind(&self) -> &CloneKind {
        match self {
            MirType::StaticStruct(struct_def) => &struct_def.static_ty.clone,
            MirType::Ref(_) => &CloneKind::Copy,
            // MirType::Primitive(_) => &CloneKind::Copy,
            _ => &CloneKind::None,
        }
    }

    /// Checks if the type is a specific concrete type.
    pub fn is<T: Reflect>(&self) -> bool {
        self.static_ty() == Some(T::TYPE)
    }

    pub fn is_supertype_of(&self, other: &Self) -> bool {
        match (self, other) {
            (MirType::None, _) => true,
            (_, MirType::None) => false,
            (MirType::Ref(pointee), MirType::Ref(other_pointee)) => {
                // don't check for supertype relationship because mutable
                // references are invariant in their pointee type
                pointee == other_pointee
            }
            (MirType::Struct(my_struct_def), MirType::Struct(other_struct_def)) => {
                Arc::ptr_eq(my_struct_def, other_struct_def)
            }
            (MirType::Array(_), MirType::Array(_)) => {
                unimplemented!("assigning entire arrays is almost surely a bug")
            }
            // (MirType::Primitive(my_ty), MirType::Primitive(other_ty)) => my_ty == other_ty,
            _ => false,
        }
    }

    pub fn new_struct(layout: Layout, fields: Vec<(usize, MirType)>) -> Self {
        Self::Struct(Arc::new(MirTypeStruct { fields, overall: layout }))
    }

    pub fn new_struct_with_static_type<T: Reflect>(fields: Vec<(usize, MirType)>) -> Self {
        Self::StaticStruct(Arc::new(MirTypeStaticStruct { static_ty: T::TYPE, fields }))
    }

    pub fn from_static_type(ty: Type) -> Self {
        // even if it's not actually a struct, a struct with inaccessible fields
        // is a good representation of the type
        Self::StaticStruct(Arc::new(MirTypeStaticStruct { static_ty: ty, fields: Vec::new() }))
    }

    pub fn ref_to(pointee: MirType) -> Self {
        Self::Ref(Box::new(pointee))
    }

    pub fn array_of(element: MirType, length: Option<usize>) -> Self {
        Self::Array(Arc::new(MirTypeArray { element, length }))
    }
}

impl fmt::Debug for MirType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MirType::Ref(pointee) => write!(f, "&{:?}", pointee),
            MirType::Struct(struct_def) => {
                let MirTypeStruct { fields, overall: _ } = struct_def.as_ref();
                write!(f, "{{")?;
                for (offset, field) in fields {
                    write!(f, " {}: {:?},", offset, field)?;
                }
                write!(f, " }}")?;
                Ok(())
            }
            MirType::Array(array) => {
                let MirTypeArray { element, length } = array.as_ref();
                if let Some(length) = length {
                    write!(f, "[{:?}; {}]", element, length)
                } else {
                    write!(f, "[{:?}; ?]", element)
                }
            }
            // MirType::Primitive(ty) => write!(f, "<prim {:?}>", ty),
            MirType::StaticStruct(struct_def) => {
                let MirTypeStaticStruct { static_ty, fields } = struct_def.as_ref();
                write!(f, "{:?}", static_ty.debug_name)?;
                if !fields.is_empty() {
                    write!(f, " + {{")?;
                    for (offset, field) in fields {
                        write!(f, " {}: {:?},", offset, field)?;
                    }
                    write!(f, " }}")?;
                }
                Ok(())
            }
            MirType::None => write!(f, "<unknown type>"),
        }
    }
}

impl MirType {
    pub fn project(&self, projection: Projection) -> &MirType {
        match projection {
            Projection::Deref => self.proj_deref(),
            Projection::Field { byte_offset } => self.proj_field(byte_offset),
            Projection::DynamicIndex(_) => self.proj_dynamic_index(),
            Projection::StaticIndex(i) => self.proj_static_index(i),
        }
    }

    pub fn proj_deref(&self) -> &MirType {
        if let MirType::Ref(pointee) = self {
            pointee
        } else {
            panic!("Cannot project type {:?} with a deref projection", self);
        }
    }

    pub fn proj_field(&self, byte_offset: usize) -> &MirType {
        if let MirType::Struct(struct_def) = self {
            let MirTypeStruct { fields, overall: _ } = struct_def.as_ref();
            let Some((_, field)) = fields.iter().find(|(offset, _)| *offset == byte_offset) else {
                panic!("Field at byte offset {} not found in type {:?}", byte_offset, self);
            };
            field
        } else {
            panic!(
                "Cannot project type {:?} with a field projection of byte offset {}",
                self, byte_offset
            );
        }
    }

    pub fn proj_static_index(&self, index: usize) -> &MirType {
        if let MirType::Array(array) = self {
            let MirTypeArray { element, length } = array.as_ref();
            if let Some(length) = length
                && index >= *length
            {
                panic!("Index {} is out of bounds for array of length {}", index, length);
            }
            element
        } else {
            panic!("Cannot project type {:?} with an index projection", self);
        }
    }

    pub fn proj_dynamic_index(&self) -> &MirType {
        if let MirType::Array(array) = self {
            let MirTypeArray { element, length: _ } = array.as_ref();
            element
        } else {
            panic!("Cannot project type {:?} with a dynamic index projection", self);
        }
    }
}

pub struct DynPtr<'a> {
    ptr: LifetimePtr<'a>,
    pointee_ty: MirType,
}

impl<'a> DynPtr<'a> {
    /// # Safety
    ///
    /// The pointer must be valid for the lifetime `'a` and the pointee type
    /// must be correct.
    pub unsafe fn new(ptr: LifetimePtr<'a>, pointee_ty: MirType) -> Self {
        Self { ptr, pointee_ty }
    }

    pub fn proj_deref(self) -> Self {
        // this checks that the pointee is itself a pointer
        let pointee_ty = self.pointee_ty.proj_deref().clone();

        // SAFETY: since we checked that the deref projection is valid,
        // the value must itself be a pointer, so we can cast it as
        // such. The fact that LifetimePtr is `repr(transparent)`
        // with a raw pointer allows us to do what is essentially a
        // transmutation.
        let ptr = *unsafe { self.ptr.cast::<LifetimePtr<'a>>() };

        Self { ptr, pointee_ty }
    }

    pub fn proj_field(self, byte_offset: usize) -> Self {
        // this checks that the pointee has a field at the given byte offset
        let pointee_ty = self.pointee_ty.proj_field(byte_offset).clone();

        // SAFETY: the pointer is valid for the lifetime `'a` and the
        // byte offset is within the bounds of the pointee type because
        // we checked with the type descriptor
        let ptr = unsafe { self.ptr.map(|ptr| ptr.byte_add(byte_offset)) };

        Self { ptr, pointee_ty }
    }

    pub fn proj_index(self, index: usize) -> Self {
        // this checks that the pointee is an array and that the index is within
        // bounds
        let pointee_ty = self.pointee_ty.proj_static_index(index).clone();

        // SAFETY: the pointer is valid for the lifetime `'a` and we checked
        // that the index is within bounds.
        let ptr = unsafe { self.ptr.map(|ptr| ptr.byte_add(index * pointee_ty.layout().size())) };

        Self { ptr, pointee_ty }
    }

    pub fn cast<T: Reflect>(self) -> &'a T {
        assert!(
            self.pointee_ty.is::<T>(),
            "type mismatch: expected {:?} but got {:?}",
            std::any::type_name::<T>(),
            self.pointee_ty
        );
        unsafe { self.ptr.cast::<T>() }
    }
}

pub struct DynPtrMut<'a> {
    ptr: LifetimePtrMut<'a>,
    pointee_ty: MirType,
}

impl<'a> DynPtrMut<'a> {
    /// # Safety
    ///
    /// The pointer must be valid for the lifetime `'a` and the pointee type
    /// must be correct.
    pub unsafe fn new(ptr: LifetimePtrMut<'a>, pointee_ty: MirType) -> Self {
        Self { ptr, pointee_ty }
    }

    pub fn proj_deref(self) -> Self {
        // this checks that the pointee is itself a pointer
        let pointee_ty = self.pointee_ty.proj_deref().clone();

        // SAFETY: since we checked that the deref projection is valid,
        // the value must itself be a pointer, so we can cast it as
        // such. The fact that LifetimePtrMut is `repr(transparent)`
        // with a raw pointer allows us to do what is essentially a
        // transmutation.
        let ptr = *unsafe { self.ptr.cast::<LifetimePtrMut<'a>>() };

        Self { ptr, pointee_ty }
    }

    pub fn proj_field(self, byte_offset: usize) -> Self {
        // this checks that the pointee has a field at the given byte offset
        let pointee_ty = self.pointee_ty.proj_field(byte_offset).clone();

        // SAFETY: the pointer is valid for the lifetime `'a` and the
        // byte offset is within the bounds of the pointee type because
        // we checked with the type descriptor
        let ptr = unsafe { self.ptr.map(|ptr| ptr.byte_add(byte_offset)) };

        Self { ptr, pointee_ty }
    }

    pub fn proj_index(self, index: usize) -> Self {
        // this checks that the pointee is an array and that the index is within
        // bounds
        let pointee_ty = self.pointee_ty.proj_static_index(index).clone();

        // SAFETY: the pointer is valid for the lifetime `'a` and we checked
        // that the index is within bounds.
        let ptr = unsafe { self.ptr.map(|ptr| ptr.byte_add(index * pointee_ty.layout().size())) };

        Self { ptr, pointee_ty }
    }

    pub fn cast<T: Reflect>(self) -> &'a mut T {
        assert!(
            self.pointee_ty.is::<T>(),
            "type mismatch: expected {:?} but got {:?}",
            std::any::type_name::<T>(),
            self.pointee_ty
        );
        unsafe { self.ptr.cast::<T>() }
    }
}

/// Indicates that the type contains a pointer that points to dynamically typed
/// data.
///
/// # Safety
///
/// Implementors must guarantee that the associated `Type` is correct, as
/// the information will be used to generate and run unsafe code.
pub unsafe trait HasDynPtr {
    /// A value of this type is sufficient to describe the type of the data
    /// pointed to by the corresponding dyn pointer
    type MetaData;

    /// Builds the MIR code to get the data pointer of the dynamically typed
    /// data, where `self_place` contains a value of `Self`.
    fn write_mir_get_data_ptr(builder: &mut FunctionBuilder, self_place: Place) -> Place;

    /// Calculates a MIR type of self from the metadata.
    fn self_mir_type_from_metadata(metadata: &Self::MetaData) -> MirType;

    /// Returns a pointer to the dynamically typed data.
    fn dyn_ptr(&self) -> DynPtr<'_>;

    /// Returns a mutable pointer to the dynamically typed data.
    fn dyn_ptr_mut(&mut self) -> DynPtrMut<'_>;
}

unsafe impl MirReflect for () {
    fn mir_type() -> MirType {
        MirType::None
    }
}

macro_rules! impl_reflect_for_primitive {
    ($ty:ty) => {
        unsafe impl MirReflect for $ty
        where
            Self: Copy,
        {
            fn mir_type() -> MirType {
                MirType::from_static_type(<$ty>::TYPE)
            }
        }
    };
}

impl_reflect_for_primitive!(bool);
impl_reflect_for_primitive!(u32);
impl_reflect_for_primitive!(f64);
impl_reflect_for_primitive!(fn(NonNull<u8>));
impl_reflect_for_primitive!(*mut u8);
