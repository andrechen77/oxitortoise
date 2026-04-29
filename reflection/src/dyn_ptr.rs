use crate::{
    DynType, Reflect,
    dyn_type::ProjectionError,
    lifetime_ptr::{LifetimePtr, LifetimePtrMut},
    mir,
};

pub struct DynPtr<'a> {
    ptr: LifetimePtr<'a>,
    pointee_ty: DynType,
}

impl<'a> DynPtr<'a> {
    /// # Safety
    ///
    /// The pointer must be valid for the lifetime `'a` and the pointee type
    /// must be correct.
    pub unsafe fn new(ptr: LifetimePtr<'a>, pointee_ty: DynType) -> Self {
        Self { ptr, pointee_ty }
    }

    pub fn proj_deref(self) -> Result<Self, ProjectionError> {
        // this checks that the pointee is itself a pointer
        let pointee_ty = self.pointee_ty.proj_deref()?.clone();

        // SAFETY: since we checked that the deref projection is valid,
        // the value must itself be a pointer, so we can cast it as
        // such. The fact that LifetimePtr is `repr(transparent)`
        // with a raw pointer allows us to do what is essentially a
        // transmutation.
        let ptr = *unsafe { self.ptr.cast::<LifetimePtr<'a>>() };

        Ok(Self { ptr, pointee_ty })
    }

    pub fn proj_field(self, byte_offset: usize) -> Result<Self, ProjectionError> {
        // this checks that the pointee has a field at the given byte offset
        let pointee_ty = self.pointee_ty.proj_field(byte_offset)?.clone();

        // SAFETY: the pointer is valid for the lifetime `'a` and the
        // byte offset is within the bounds of the pointee type because
        // we checked with the type descriptor
        let ptr = unsafe { self.ptr.map(|ptr| ptr.byte_add(byte_offset)) };

        Ok(Self { ptr, pointee_ty })
    }

    pub fn proj_index(self, index: usize) -> Result<Self, ProjectionError> {
        // this checks that the pointee is an array and that the index is within
        // bounds
        let pointee_ty = self.pointee_ty.proj_static_index(index)?.clone();

        // SAFETY: the pointer is valid for the lifetime `'a` and we checked
        // that the index is within bounds.
        let ptr = unsafe { self.ptr.map(|ptr| ptr.byte_add(index * pointee_ty.layout().size())) };

        Ok(Self { ptr, pointee_ty })
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
    pointee_ty: DynType,
}

impl<'a> DynPtrMut<'a> {
    /// # Safety
    ///
    /// The pointer must be valid for the lifetime `'a` and the pointee type
    /// must be correct.
    pub unsafe fn new(ptr: LifetimePtrMut<'a>, pointee_ty: DynType) -> Self {
        Self { ptr, pointee_ty }
    }

    pub fn proj_deref(self) -> Result<Self, ProjectionError> {
        // this checks that the pointee is itself a pointer
        let pointee_ty = self.pointee_ty.proj_deref()?.clone();

        // SAFETY: since we checked that the deref projection is valid,
        // the value must itself be a pointer, so we can cast it as
        // such. The fact that LifetimePtrMut is `repr(transparent)`
        // with a raw pointer allows us to do what is essentially a
        // transmutation.
        let ptr = *unsafe { self.ptr.cast::<LifetimePtrMut<'a>>() };

        Ok(Self { ptr, pointee_ty })
    }

    pub fn proj_field(self, byte_offset: usize) -> Result<Self, ProjectionError> {
        // this checks that the pointee has a field at the given byte offset
        let pointee_ty = self.pointee_ty.proj_field(byte_offset)?.clone();

        // SAFETY: the pointer is valid for the lifetime `'a` and the
        // byte offset is within the bounds of the pointee type because
        // we checked with the type descriptor
        let ptr = unsafe { self.ptr.map(|ptr| ptr.byte_add(byte_offset)) };

        Ok(Self { ptr, pointee_ty })
    }

    pub fn proj_index(self, index: usize) -> Result<Self, ProjectionError> {
        // this checks that the pointee is an array and that the index is within
        // bounds
        let pointee_ty = self.pointee_ty.proj_static_index(index)?.clone();

        // SAFETY: the pointer is valid for the lifetime `'a` and we checked
        // that the index is within bounds.
        let ptr = unsafe { self.ptr.map(|ptr| ptr.byte_add(index * pointee_ty.layout().size())) };

        Ok(Self { ptr, pointee_ty })
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

// TODO this trait is really not necessary. it could just be methods on each
// type itself.
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
    fn write_mir_get_data_ptr(
        builder: &mut mir::FunctionBuilder,
        self_place: mir::Place,
    ) -> mir::Place;

    /// Calculates a MIR type of self from the metadata.
    fn self_mir_type_from_metadata(metadata: &Self::MetaData) -> DynType;

    /// Returns a pointer to the dynamically typed data.
    fn dyn_ptr(&self) -> DynPtr<'_>;

    /// Returns a mutable pointer to the dynamically typed data.
    fn dyn_ptr_mut(&mut self) -> DynPtrMut<'_>;
}
