use std::{alloc::Layout, fmt, ptr::NonNull, sync::Arc};

use tracing::{trace, warn};

use crate::{CloneKind, Reflect, StaticType, mir::Projection};

/// A trait to indicate how accesses into values of this type can be generated.
///
/// # Safety
///
/// Implementors must guarantee that the associated `create_dyn_type` is
/// correct, as the information will be used to generate and run unsafe code.
pub unsafe trait CreateDynType {
    /// This should only be called once, and stored for future queries about the
    /// same type.
    fn create_dyn_type() -> DynType;
}

#[derive(Clone, Default, PartialEq, Eq)]
pub enum DynType {
    /// A non-owning, mutable reference to the specified type. Equivalent to
    /// `&mut T` but doesn't have a uniqueness guarantee.
    Ref(Box<DynType>),
    /// An aggregate type with the specified fields.
    Struct(Arc<DynTypeStruct>),
    /// An array having an element type that satisfies the given assertion. If
    /// the length is specified, then the array has exactly that many elements,
    /// otherwise it has a statically unknown length.
    Array(Arc<DynTypeArray>),
    /// A statically known type
    StaticStruct(Arc<DynTypeStaticStruct>),
    /// We know nothing about this type.
    #[default]
    None,
}

#[derive(PartialEq, Eq)]
pub struct DynTypeStruct {
    /// The fields of the type. This may not be a complete list.
    pub fields: Vec<(usize, DynType)>,
    pub overall: Layout,
}

#[derive(PartialEq, Eq)]
pub struct DynTypeArray {
    pub element: DynType,
    pub length: Option<usize>,
}

#[derive(PartialEq, Eq)]
pub struct DynTypeStaticStruct {
    pub static_ty: StaticType,
    /// Any additional information about the fields of this struct.
    pub fields: Vec<(usize, DynType)>,
}

impl DynType {
    pub fn layout(&self) -> Layout {
        match self {
            DynType::Ref(_) => Layout::new::<*const u8>(),
            DynType::Struct(struct_def) => struct_def.overall,
            DynType::Array(_) => {
                unimplemented!(
                    "would use the Layout::repeat function on the element layout to get the layout of the whole array"
                )
            }
            DynType::StaticStruct(struct_def) => struct_def.static_ty.layout.unwrap(),
            DynType::None => panic!("Cannot get layout"),
        }
    }

    pub fn static_ty(&self) -> Option<StaticType> {
        trace!("getting static_ty of: {:?}", self);
        match self {
            DynType::StaticStruct(struct_def) => Some(struct_def.static_ty),
            _ => None,
        }
    }

    pub fn has_drop_fn(&self) -> bool {
        match self {
            DynType::StaticStruct(struct_def) => struct_def.static_ty.drop_fn.is_some(),
            DynType::Struct(_struct_def) => {
                unimplemented!("We shouldn't be dropping custom structs...")
            }
            _ => false,
        }
    }

    pub fn clone_kind(&self) -> &CloneKind {
        match self {
            DynType::StaticStruct(struct_def) => &struct_def.static_ty.clone,
            DynType::Ref(_) => &CloneKind::Copy,
            // MirType::Primitive(_) => &CloneKind::Copy,
            _ => &CloneKind::None,
        }
    }

    /// Checks if the type is a specific concrete type.
    pub fn is<T: Reflect>(&self) -> bool {
        self.static_ty() == Some(T::STATIC_TYPE)
    }

    pub fn is_supertype_of(&self, other: &Self) -> bool {
        match (self, other) {
            (DynType::None, _) => true,
            (_, DynType::None) => false,
            (DynType::Ref(pointee), DynType::Ref(other_pointee)) => {
                // don't check for supertype relationship because mutable
                // references are invariant in their pointee type
                pointee == other_pointee
            }
            (DynType::Struct(my_struct_def), DynType::Struct(other_struct_def)) => {
                Arc::ptr_eq(my_struct_def, other_struct_def)
            }
            (DynType::Array(_), DynType::Array(_)) => {
                unimplemented!("assigning entire arrays is almost surely a bug")
            }
            (DynType::StaticStruct(my_struct_def), DynType::StaticStruct(other_struct_def)) => {
                Arc::ptr_eq(my_struct_def, other_struct_def)
            }
            _ => false,
        }
    }

    pub fn new_struct(layout: Layout, fields: Vec<(usize, DynType)>) -> Self {
        Self::Struct(Arc::new(DynTypeStruct { fields, overall: layout }))
    }

    pub fn new_struct_with_static_type<T: Reflect>(fields: Vec<(usize, DynType)>) -> Self {
        Self::StaticStruct(Arc::new(DynTypeStaticStruct { static_ty: T::STATIC_TYPE, fields }))
    }

    pub fn ref_to(pointee: DynType) -> Self {
        Self::Ref(Box::new(pointee))
    }

    pub fn array_of(element: DynType, length: Option<usize>) -> Self {
        Self::Array(Arc::new(DynTypeArray { element, length }))
    }
}

impl fmt::Debug for DynType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DynType::Ref(pointee) => write!(f, "&{:?}", pointee),
            DynType::Struct(struct_def) => {
                let DynTypeStruct { fields, overall: _ } = struct_def.as_ref();
                write!(f, "{{")?;
                for (offset, field) in fields {
                    write!(f, " {}: {:?},", offset, field)?;
                }
                write!(f, " }}")?;
                Ok(())
            }
            DynType::Array(array) => {
                let DynTypeArray { element, length } = array.as_ref();
                if let Some(length) = length {
                    write!(f, "[{:?}; {}]", element, length)
                } else {
                    write!(f, "[{:?}; ?]", element)
                }
            }
            DynType::StaticStruct(struct_def) => {
                let DynTypeStaticStruct { static_ty, fields } = struct_def.as_ref();
                write!(f, "{}", static_ty.debug_name)?;
                if !fields.is_empty() {
                    write!(f, " + {{")?;
                    for (offset, field) in fields {
                        write!(f, " {}: {:?},", offset, field)?;
                    }
                    write!(f, " }}")?;
                }
                Ok(())
            }
            DynType::None => write!(f, "<unknown type>"),
        }
    }
}

impl DynType {
    pub fn project(&self, projection: Projection) -> Result<&DynType, ProjectionError> {
        match projection {
            Projection::Deref => self.proj_deref(),
            Projection::Field { byte_offset } => self.proj_field(byte_offset),
            Projection::DynamicIndex(_) => self.proj_dynamic_index(),
            Projection::StaticIndex(i) => self.proj_static_index(i),
        }
    }

    pub fn proj_deref(&self) -> Result<&DynType, ProjectionError> {
        if let DynType::Ref(pointee) = self {
            Ok(pointee)
        } else {
            warn!("Cannot project type {:?} with a deref projection", self);
            Err(ProjectionError)
        }
    }

    pub fn proj_field(&self, byte_offset: usize) -> Result<&DynType, ProjectionError> {
        let fields = match self {
            DynType::Struct(struct_def) => &struct_def.fields,
            DynType::StaticStruct(struct_def) => &struct_def.fields,
            _ => {
                warn!(
                    "Cannot project type {:?} with a field projection of byte offset {}",
                    self, byte_offset
                );
                return Err(ProjectionError);
            }
        };
        let Some((_, ty)) = fields.iter().find(|(offset, _)| *offset == byte_offset) else {
            warn!("Field at byte offset {} not found in type {:?}", byte_offset, self);
            return Err(ProjectionError);
        };
        Ok(ty)
    }

    pub fn proj_static_index(&self, index: usize) -> Result<&DynType, ProjectionError> {
        if let DynType::Array(array) = self {
            let DynTypeArray { element, length } = array.as_ref();
            if let Some(length) = length
                && index >= *length
            {
                warn!("Index {} is out of bounds for array of length {}", index, length);
            }
            Ok(element)
        } else {
            warn!("Cannot project type {:?} with an index projection", self);
            Err(ProjectionError)
        }
    }

    pub fn proj_dynamic_index(&self) -> Result<&DynType, ProjectionError> {
        if let DynType::Array(array) = self {
            let DynTypeArray { element, length: _ } = array.as_ref();
            Ok(element)
        } else {
            warn!("Cannot project type {:?} with a dynamic index projection", self);
            Err(ProjectionError)
        }
    }
}

pub struct ProjectionError;

unsafe impl<T> CreateDynType for &mut T
where
    T: Reflect,
{
    fn create_dyn_type() -> DynType {
        DynType::ref_to(T::dyn_type())
    }
}

unsafe impl<T> CreateDynType for &T
where
    T: Reflect,
{
    fn create_dyn_type() -> DynType {
        DynType::ref_to(T::dyn_type())
    }
}

unsafe impl CreateDynType for () {
    fn create_dyn_type() -> DynType {
        DynType::new_struct_with_static_type::<()>(vec![])
    }
}

macro_rules! impl_reflect_for_primitive {
    ($ty:ty) => {
        unsafe impl CreateDynType for $ty
        where
            Self: Copy,
        {
            fn create_dyn_type() -> DynType {
                // even if it's not actually a struct, a struct with
                // inaccessible fields is a good representation of the type
                DynType::new_struct_with_static_type::<Self>(vec![])
            }
        }
    };
}

impl_reflect_for_primitive!(bool);
impl_reflect_for_primitive!(u32);
impl_reflect_for_primitive!(f64);
impl_reflect_for_primitive!(fn(NonNull<u8>));
impl_reflect_for_primitive!(*mut u8);
