use std::{alloc::Layout, sync::Arc};

use crate::{
    mir::{Place, Projection, builder::FunctionBuilder},
    util::{
        lifetime_ptr::{LifetimePtr, LifetimePtrMut},
        reflection::{Reflect, Type},
    },
};

pub type MirType = Arc<MirTypeInfo>;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MirTypeInfo {
    /// If it exists, the static type that this MirType represents.
    pub static_ty: Option<Type>,
    /// Additional information about the contents of the type and how it can be
    /// accessed.
    pub contents: MirTypeContents,
}

/// Represents a description of how a type is stored in memory and how it can
/// be accessed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum MirTypeContents {
    /// The value is a pointer to a type that satisfies the given assertion.
    Ptr(MirType),
    /// The value is a struct that has fields at the specified byte offsets.
    Struct { fields: Vec<(usize, MirType)>, overall: Layout, fields_are_complete: bool },
    /// The value is a array having an element type that satisfies the given
    /// assertion. If the length is specified, then the array has exactly that
    /// many elements, otherwise it has a statically unknown length.
    Array { element: MirType, length: Option<usize> },
    /// The value is a primitive type.
    Prim(lir::ValType),
    /// No assertion.
    #[default]
    None,
}

impl MirTypeInfo {
    pub fn layout(&self) -> Layout {
        if let Some(static_ty) = self.static_ty
            && let Some(layout) = static_ty.layout
        {
            return layout;
        }
        match &self.contents {
            MirTypeContents::Ptr(_) => Layout::new::<*const u8>(),
            MirTypeContents::Struct { overall, .. } => *overall,
            MirTypeContents::Array { .. } => {
                unimplemented!(
                    "would use the Layout::repeat function on the element layout to get the layout of the whole array"
                )
            }
            MirTypeContents::Prim(_ty) => todo!("TODO get the layout of a primitive type"),
            MirTypeContents::None => {
                panic!("Cannot get layout")
            }
        }
    }

    /// Checks if the type is a specific concrete type.
    pub fn is<T: Reflect>(&self) -> bool {
        if let Some(static_ty) = self.static_ty { static_ty.is::<T>() } else { false }
    }

    /// Asserts that the value is a specific concrete type, and panics if it is
    /// not.
    pub fn assert_is<T: Reflect>(&self) {
        assert!(self.is::<T>(), "Expected type {:?} but got {:?}", T::TYPE, self);
    }
}

impl MirTypeContents {
    pub fn project(&self, projection: Projection) -> &MirType {
        use MirTypeContents as M;
        match (self, projection) {
            (M::Ptr(pointee), Projection::Deref) => pointee,
            (M::Struct { fields, .. }, Projection::Field { byte_offset }) => {
                let Some((_, field)) = fields.iter().find(|(offset, _)| *offset == byte_offset)
                else {
                    panic!("Field at byte offset {} not found", byte_offset);
                };
                field
            }
            (M::Array { element, length: _ }, Projection::DynamicIndex(_index)) => element,
            (desc, projection) => {
                panic!(
                    "Cannot project memory descriptor {:?} with projection: {:?}",
                    desc, projection
                )
            }
        }
    }

    pub fn proj_deref(&self) -> &MirType {
        if let MirTypeContents::Ptr(pointee) = self {
            pointee
        } else {
            panic!("Cannot project type {:?} with a deref projection", self);
        }
    }

    pub fn proj_field(&self, byte_offset: usize) -> &MirType {
        if let MirTypeContents::Struct { fields, .. } = self {
            let Some((_, field)) = fields.iter().find(|(offset, _)| *offset == byte_offset) else {
                panic!("Field at byte offset {} not found", byte_offset);
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
        if let MirTypeContents::Array { element, length } = self {
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
        if let MirTypeContents::Array { element, length: _ } = self {
            element
        } else {
            panic!("Cannot project type {:?} with a dynamic index projection", self);
        }
    }
}

impl MirTypeInfo {
    pub fn ptr_to(pointee: MirType) -> MirType {
        Arc::new(MirTypeInfo { static_ty: None, contents: MirTypeContents::Ptr(pointee) })
    }

    pub fn array_of(element: MirType, length: Option<usize>) -> MirType {
        Arc::new(MirTypeInfo {
            static_ty: None,
            contents: MirTypeContents::Array { element, length },
        })
    }

    pub fn struct_with_some_fields(layout: Layout, fields: Vec<(usize, MirType)>) -> MirType {
        Arc::new(MirTypeInfo {
            static_ty: None,
            contents: MirTypeContents::Struct {
                fields,
                overall: layout,
                fields_are_complete: false,
            },
        })
    }

    pub fn struct_with_all_fields(layout: Layout, fields: Vec<(usize, MirType)>) -> MirType {
        Arc::new(MirTypeInfo {
            static_ty: None,
            contents: MirTypeContents::Struct {
                fields,
                overall: layout,
                fields_are_complete: true,
            },
        })
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
        let pointee_ty = self.pointee_ty.contents.proj_deref().clone();

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
        let pointee_ty = self.pointee_ty.contents.proj_field(byte_offset).clone();

        // SAFETY: the pointer is valid for the lifetime `'a` and the
        // byte offset is within the bounds of the pointee type because
        // we checked with the type descriptor
        let ptr = unsafe { self.ptr.map(|ptr| ptr.byte_add(byte_offset)) };

        Self { ptr, pointee_ty }
    }

    pub fn proj_index(self, index: usize) -> Self {
        // this checks that the pointee is an array and that the index is within
        // bounds
        let pointee_ty = self.pointee_ty.contents.proj_static_index(index).clone();

        // SAFETY: the pointer is valid for the lifetime `'a` and we checked
        // that the index is within bounds.
        let ptr = unsafe { self.ptr.map(|ptr| ptr.byte_add(index * pointee_ty.layout().size())) };

        Self { ptr, pointee_ty }
    }

    pub fn cast<T: Reflect>(self) -> &'a T {
        self.pointee_ty.assert_is::<T>();
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
        let pointee_ty = self.pointee_ty.contents.proj_deref().clone();

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
        let pointee_ty = self.pointee_ty.contents.proj_field(byte_offset).clone();

        // SAFETY: the pointer is valid for the lifetime `'a` and the
        // byte offset is within the bounds of the pointee type because
        // we checked with the type descriptor
        let ptr = unsafe { self.ptr.map(|ptr| ptr.byte_add(byte_offset)) };

        Self { ptr, pointee_ty }
    }

    pub fn proj_index(self, index: usize) -> Self {
        // this checks that the pointee is an array and that the index is within
        // bounds
        let pointee_ty = self.pointee_ty.contents.proj_static_index(index).clone();

        // SAFETY: the pointer is valid for the lifetime `'a` and we checked
        // that the index is within bounds.
        let ptr = unsafe { self.ptr.map(|ptr| ptr.byte_add(index * pointee_ty.layout().size())) };

        Self { ptr, pointee_ty }
    }

    pub fn cast<T: Reflect>(self) -> &'a mut T {
        self.pointee_ty.assert_is::<T>();
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
