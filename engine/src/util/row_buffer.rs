//! A buffer of rows, each of which has a fixed schema.
//!
//! The schema describes the fields in each row, and the layout of the fields
//! in memory. Any field may be present or absent. If the field is absent,
//! its memory is all zeros.
//!
use std::{
    alloc::{Layout, alloc_zeroed, dealloc},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::util::reflection::{ConcreteTy, Reflect};

#[derive(Debug)]
#[repr(C)]
pub struct AlignedBytes {
    data: NonNull<u8>,
    layout: Layout,
}

impl AlignedBytes {
    pub fn new(layout: Layout) -> Self {
        // if the capacity is zero then it should not actually allocate. but
        // we don't need to be fancy like std and have a dangling pointer.
        // we can just panic

        if layout.size() == 0 {
            panic!("capacity must be greater than 0");
        }
        // SAFETY: we checked that the size is greater than 0
        let ptr =
            NonNull::new(unsafe { alloc_zeroed(layout) }).expect("allocation should not fail");
        Self { data: ptr, layout }
    }

    /// Increases the size of the buffer to be at least `new_size`. New bytes
    /// are initialized to zero. This function will not shrink the buffer.
    pub fn resize(&mut self, new_size: usize) {
        if new_size <= self.layout.size() {
            return;
        }

        let new_layout = Layout::from_size_align(new_size, self.layout.align())
            .expect("alignment is already known to be correct, and we do not handle the case where new_size is too big");
        let mut new = Self::new(new_layout);
        // possible optimization: the `new` function automatically zeroes the
        // new memory, but we don't need to do that if we're just going to
        // copy from the old memory. maybe the compiler is smart enough to do
        // this automatically though.
        new[..self.layout.size()].copy_from_slice(&self[..]);
        *self = new;
        // the old memory is dropped here
    }

    pub fn layout(&self) -> Layout {
        self.layout
    }
}

impl Deref for AlignedBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        // SAFETY: we own this data and know it exists
        unsafe { NonNull::slice_from_raw_parts(self.data, self.layout.size()).as_ref() }
    }
}

impl DerefMut for AlignedBytes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: we own this data and know it exists
        unsafe { NonNull::slice_from_raw_parts(self.data, self.layout.size()).as_mut() }
    }
}

impl Drop for AlignedBytes {
    fn drop(&mut self) {
        // SAFETY: we own this data and know it is currently allocated
        unsafe { dealloc(self.data.as_ptr(), self.layout) };
    }
}

#[derive(Debug)]
pub struct RowSchema {
    /// The length of the occupancy bitfield. If schema has fields that are
    /// always present, then this is zero.
    occupancy_bitfield_len: usize,
    fields: Vec<RowSchemaField>,
    /// The layout of an entire row. This includes padding to ensure that the
    /// next row is aligned.
    layout: Layout,
}

// For safety this must be correct at all times.
#[derive(Debug)]
pub struct RowSchemaField {
    pub offset: usize,
    pub r#type: ConcreteTy,
    pub size: usize,
}

impl RowSchema {
    pub fn new(types_and_layouts: &[ConcreteTy], occupancy_bitfield: bool) -> Self {
        let mut fields = Vec::new();

        // find the length of the bitfield to hold whether each column is
        // present
        let occupancy_bitfield_len =
            if occupancy_bitfield { types_and_layouts.len().div_ceil(8) } else { 0 };

        // find the offsets of each field
        // https://doc.rust-lang.org/std/alloc/struct.Layout.html#method.extend
        let mut overall_layout = Layout::from_size_align(occupancy_bitfield_len, 1)
            .expect("this layout should be valid");
        for &field_type in types_and_layouts {
            // safety relies on the fact that this function returns a correct
            // answer
            let field_layout = field_type.info().layout;

            // append this field to the current layout
            let (new_layout, offset) = overall_layout
                .extend(field_layout)
                .expect("records should not be so big as to overflow");
            fields.push(RowSchemaField { offset, r#type: field_type, size: field_layout.size() });
            overall_layout = new_layout;
        }
        // add padding at the end to align with itself
        overall_layout = overall_layout.pad_to_align();

        Self { occupancy_bitfield_len, fields, layout: overall_layout }
    }

    pub fn can_reinterpret_as<T: Reflect>(&self) -> bool {
        // must have no occupancy bitfield (all fields always present)
        if self.occupancy_bitfield_len != 0 {
            return false;
        }

        // must have exactly one field
        if self.fields.len() != 1 {
            return false;
        }
        let field = &self.fields[0];

        // field must be of type T and at offset 0
        if field.r#type != T::CONCRETE_TY || field.offset != 0 {
            return false;
        }

        // overall layout must match T's layout
        let t_layout = Layout::new::<T>();
        self.layout == t_layout
    }

    pub fn field(&self, field_idx: usize) -> &RowSchemaField {
        &self.fields[field_idx]
    }

    pub fn stride(&self) -> usize {
        self.layout.size()
    }
}

pub struct Row<'a, B: 'a> {
    schema: &'a RowSchema,
    /// Safety: must point to a valid row in the row buffer
    data: B,
}

impl<'a, B: AsRef<[u8]> + 'a> Row<'a, B> {
    pub fn get_ptr(&self, field_idx: usize) -> *const () {
        let field_offset = self.schema.fields[field_idx].offset;
        let base_ptr = self.data.as_ref() as *const [u8] as *const ();
        // SAFETY: the offset is into an allocated object
        unsafe { base_ptr.byte_add(field_offset) }
    }

    pub fn get<T: Reflect>(&self, field_idx: usize) -> Option<&'a T> {
        // verify that the type tag matches
        assert_eq!(self.schema.fields[field_idx].r#type, T::CONCRETE_TY, "type mismatch");

        let ptr = self.get_ptr(field_idx);
        // check if the field is actually present
        if self.has_field(field_idx) {
            // SAFETY: self.data points to a row in the row buffer with the
            // schema described by self.schema, which indicates that this offset
            // has the given type, and the field is present because we checked
            // with self.has_field()
            unsafe { Some(&*(ptr as *const T)) }
        } else {
            None
        }
    }

    pub fn has_field(&self, field_idx: usize) -> bool {
        // handle the case where the occupancy bitfield is omitted due to
        // always-present fields
        if self.schema.occupancy_bitfield_len == 0 {
            return true;
        }

        // check whether the bit at `field_idx` is set
        let byte_offset = field_idx / 8;
        let bit_offset = field_idx % 8;
        self.data.as_ref()[byte_offset] & (1 << bit_offset) != 0
    }
}

impl<'a, B: AsRef<[u8]> + AsMut<[u8]> + 'a> Row<'a, B> {
    pub fn get_ptr_mut(&mut self, field_idx: usize) -> *mut u8 {
        let field_offset = self.schema.fields[field_idx].offset;
        let base_ptr = self.data.as_mut() as *mut [u8] as *mut u8;
        // SAFETY: the offset is into an allocated object
        unsafe { base_ptr.byte_add(field_offset) }
    }

    pub fn get_mut<T: Reflect>(&mut self, field_idx: usize) -> Option<&'a mut T> {
        // same implementation as self.get
        assert_eq!(self.schema.fields[field_idx].r#type, T::CONCRETE_TY, "type mismatch");

        let ptr = self.get_ptr_mut(field_idx);
        if self.has_field(field_idx) {
            // SAFETY: see self.get's corresponding comment
            unsafe { Some(&mut *(ptr as *mut T)) }
        } else {
            None
        }
    }

    pub fn insert<T: Reflect>(&mut self, field_idx: usize, value: T) {
        assert_eq!(self.schema.fields[field_idx].r#type, T::CONCRETE_TY, "type mismatch");

        // if the field is already present, panic. we could technically just
        // drop the existing value, or return the new value to the caller, but
        // in the context of this codebase I don't see a reason the user might
        // want to do that
        if self.has_field(field_idx) {
            panic!("field at index {} is already present", field_idx);
        }

        // write the value to the row buffer
        let ptr = self.get_ptr_mut(field_idx) as *mut T;
        // SAFETY: the type tag matches, and the schema has the correct
        // type and size for the field
        unsafe { std::ptr::write(ptr, value) };

        // set the occupancy bit
        self.mark_present(field_idx);
    }

    /// Marks some value in the field as present without any initialization. If
    /// the field is not present, then the field's memory is already zeroed out,
    /// which should be valid for the type at the given index.
    pub fn insert_zeroable(&mut self, field_idx: usize) {
        let type_info = self.schema.fields[field_idx].r#type.info();
        if !type_info.is_zeroable {
            panic!("field at index {} is not zeroable", field_idx);
        }

        // set the occupancy bit. we don't need to do anything else because we
        // whether the field is absent (in which case it is all zeros due to
        // mark_absent) or present, it is valid
        self.mark_present(field_idx);
    }

    pub fn take<T: Reflect>(&mut self, field_idx: usize) -> Option<T> {
        assert_eq!(self.schema.fields[field_idx].r#type, T::CONCRETE_TY, "type mismatch");

        if self.schema.occupancy_bitfield_len == 0 {
            panic!("cannot take a field if the occupancy bitfield is omitted");
        }

        if !self.has_field(field_idx) {
            return None;
        }

        let ptr = self.get_ptr_mut(field_idx) as *mut T;
        // SAFETY: the type tag matches, and the schema has the correct type and
        // size for the field. we also checked that the field is actually
        // present
        let value = unsafe { std::ptr::read(ptr) };

        // this also zeros out the memory
        self.mark_absent(field_idx);
        Some(value)
    }

    /// # Safety
    ///
    /// This must not be called twice on a row if the row schema indicates that
    /// fields are always present.
    unsafe fn drop_all_fields(mut self) {
        // run the destructor for each field that is present
        for field_idx in 0..self.schema.fields.len() {
            if !self.has_field(field_idx) {
                continue;
            }

            let field_type = self.schema.fields[field_idx].r#type;
            let drop_fn = field_type.info().drop_fn;
            // SAFETY: we used the type tag for this field to get the right
            // function to call to drop the value. we know this value is
            // actually present. this value will not be read again because the
            // field will be marked as not present in the following line
            unsafe { drop_fn(self.get_ptr_mut(field_idx)) };
            if self.schema.occupancy_bitfield_len != 0 {
                self.mark_absent(field_idx);
            }
        }
    }

    fn mark_present(&mut self, field_idx: usize) {
        // handle the case where the occupancy bitfield is omitted due to
        // always-present fields
        if self.schema.occupancy_bitfield_len == 0 {
            return;
        }

        let byte_offset = field_idx / 8;
        let bit_offset = field_idx % 8;
        self.data.as_mut()[byte_offset] |= 1 << bit_offset;
    }

    /// Marks the field at `field_idx` as absent in the occupancy bitfield.
    /// Also zeroes the field's memory.
    fn mark_absent(&mut self, field_idx: usize) {
        // handle the case where the occupancy bitfield is omitted due to
        // always-present fields
        if self.schema.occupancy_bitfield_len == 0 {
            panic!("cannot mark a field as absent if the occupancy bitfield is omitted");
        }

        let byte_offset = field_idx / 8;
        let bit_offset = field_idx % 8;
        self.data.as_mut()[byte_offset] &= !(1 << bit_offset);

        // zero the field's memory
        let RowSchemaField { offset, size, .. } = self.schema.fields[field_idx];
        self.data.as_mut()[offset..offset + size].fill(0);
    }
}

#[unsafe(no_mangle)]
static SIZE_OF_ROW_BUFFER: usize = size_of::<RowBuffer>();

#[derive(Debug)]
#[repr(C)]
pub struct RowBuffer {
    /// The bytes for the row data.
    bytes: AlignedBytes,
    /// The structure of each row in the array.
    schema: RowSchema,
}

impl RowBuffer {
    fn make_byte_buffer(schema: &RowSchema, num_rows: usize) -> AlignedBytes {
        //  the layout of all the rows at once
        /*
        // only usable once alloc_layout_extra is stabilized
        let (total_layout, stride) = schema
            .layout
            .repeat(num_rows)
            .expect("if we overflow on a layout calculate we're in bigger trouble");
        */

        let mut total_layout = schema.layout;
        let mut stride = None;
        for _ in 1..num_rows {
            let (new_layout, offset_of_last_elem) = total_layout
                .extend(schema.layout)
                .expect("if we overflow on layout calculation we're in bigger trouble");
            total_layout = new_layout;
            if let Some(stride) = stride {
                assert!(offset_of_last_elem % stride == 0);
            } else {
                stride = Some(offset_of_last_elem);
            }
        }
        total_layout = total_layout.pad_to_align(); // unnecessary but just in case

        // this must be true because we use layout size as the stride in
        // other methods
        assert!(schema.layout.size() == stride.unwrap());

        AlignedBytes::new(total_layout)
    }

    /// Creates a RowBuffer with some initial number of rows.
    /// See [`Self::new_with_capacity`] for more information.
    pub fn new(schema: RowSchema) -> Self {
        // arbitrary initial size
        let num_rows = 100;
        Self::new_with_capacity(schema, num_rows)
    }

    /// When the [`RowSchema`] indicates that all fields are always present,
    /// then the fields must be valid at the zero bit pattern.
    pub fn new_with_capacity(schema: RowSchema, num_rows: usize) -> Self {
        // for safety, check that if the fields are assumed to be always,
        // then they must already be valid at the zero bit pattern
        if schema.occupancy_bitfield_len == 0 {
            for (field_idx, field) in schema.fields.iter().enumerate() {
                if !field.r#type.info().is_zeroable {
                    panic!(
                        "cannot initialize an always-present row buffer when field at index {} is not zeroable",
                        field_idx
                    );
                }
            }
        }

        let bytes = Self::make_byte_buffer(&schema, num_rows);
        Self { schema, bytes }
    }

    pub fn num_rows(&self) -> usize {
        self.bytes.len() / self.schema.layout.size()
    }

    pub fn schema(&self) -> &RowSchema {
        &self.schema
    }

    /// # Panics
    ///
    /// Panics if the row index is out of bounds. Run [`Self::ensure_capacity`]
    /// first if you are not sure that the row exists.
    pub fn row(&self, row_idx: usize) -> Row<'_, &[u8]> {
        let offset = row_idx * self.schema.layout.size();
        let stride = self.schema.layout.size();
        let data = &self.bytes[offset..offset + stride];
        Row { schema: &self.schema, data }
    }

    /// # Panics
    ///
    /// Panics if the row index is out of bounds. Run [`Self::ensure_capacity`]
    /// first if you are not sure that the row exists.
    pub fn row_mut(&mut self, row_idx: usize) -> Row<'_, &mut [u8]> {
        let offset = row_idx * self.schema.layout.size();
        let stride = self.schema.layout.size();
        let data = &mut self.bytes[offset..offset + stride];
        Row { schema: &self.schema, data }
    }

    /// Resizes the row buffer to be able to hold at least `num_rows` rows.
    ///
    /// This function will not shrink the buffer.
    pub fn ensure_capacity(&mut self, num_rows: usize) {
        self.bytes.resize(num_rows * self.schema.layout.size());
    }

    // TODO(wishlist) add functionality for unchecked insert/remove rows

    /// Remaps all the rows the buffer to a new schema. [`map_fn`] is called
    /// with a reference to the old row and the new row.
    pub fn change_schema<F>(&mut self, new_schema: RowSchema, mut map_fn: F)
    where
        F: FnMut(Row<'_, &mut [u8]>, Row<'_, &mut [u8]>),
    {
        let num_rows = self.num_rows();
        let mut new_buffer = Self::new_with_capacity(new_schema, num_rows);

        for row_idx in 0..num_rows {
            let old_row = self.row_mut(row_idx);
            let new_row = new_buffer.row_mut(row_idx);
            map_fn(old_row, new_row);
        }

        *self = new_buffer;
        // old memory is dropped here
    }

    /// Takes all the data as an array of `T`, panicking if the memory cannot be
    /// correctly reinterpreted. The existing row buffer is replaced with all
    /// zeroes.
    ///
    /// For simplicity of implementation this only needs to work if the schema
    /// has a single field of type `T`, and has the same alignment/stride as `T`.
    pub fn take_array<T: Copy + Reflect>(&mut self) -> Array<T> {
        if !self.schema.can_reinterpret_as::<T>() {
            panic!(
                "cannot reinterpret the row buffer as an array of type {}",
                std::any::type_name::<T>()
            );
        }

        // create a zero-initialized buffer with the same layout to replace the
        // bytes we're about to steal. we know that zero-initialization works
        // because the being able to reinterpret as T means there is no
        // occupancy bitfield which means all fields are present, and according
        // to the RowBuffer constructor this is only allowed if all fields are
        // zeroable.
        let new_bytes = AlignedBytes::new(self.bytes.layout());
        let old_bytes = std::mem::replace(&mut self.bytes, new_bytes);
        Array { bytes: old_bytes, _phantom: PhantomData }
    }

    pub fn as_mut_array<T: Copy + Reflect>(&mut self) -> &mut [T] {
        if !self.schema.can_reinterpret_as::<T>() {
            panic!(
                "cannot reinterpret the row buffer as an array of type {}",
                std::any::type_name::<T>()
            );
        }

        let start = self.bytes.deref_mut().as_mut_ptr().cast::<T>();
        let len = self.num_rows();
        // SAFETY: the bytes are guaranteed to hold valid T values
        unsafe { std::slice::from_raw_parts_mut(start, len) }
    }

    /// Drops all fields in all rows.
    pub fn clear(&mut self) {
        if self.schema.occupancy_bitfield_len == 0 {
            panic!("cannot clear a row buffer where all fields must be always present");
        }

        // SAFETY: we know that the schema does not indicate that all fields
        // are always present, so we can drop all fields
        unsafe { self.drop_all_fields() };
    }

    /// Drops all fields in all rows
    ///
    /// # Safety
    ///
    /// If this row buffer has always-present fields, then this function
    /// must not be called more than once.
    unsafe fn drop_all_fields(&mut self) {
        for row_idx in 0..self.num_rows() {
            let row = self.row_mut(row_idx);
            // SAFETY: we know that the schema does not indicate that all fields
            // are always present, so we can drop all fields
            unsafe { row.drop_all_fields() };
        }
    }
}

impl Drop for RowBuffer {
    fn drop(&mut self) {
        // SAFETY: this RowBuffer will never be used again so we can drop all
        // fields
        unsafe { self.drop_all_fields() };
    }
}

// The requirement of T: Copy is just to make the implementation simpler since
// the engine doesn't currently need to support types other than pure numbers.
// If it is not Copy, then we would need to drop all the elements of T when the
// array is dropped.

// We might be able to remove this type in favor of a simple Box<[T]> if we can
// figure out how to make it call the right drop function; until then this is
// necessary to deallocate the AlignedBytes.

pub struct Array<T: Copy> {
    /// The bytes holding the data of the array. It is guaranteed to hold
    /// valid T values.
    bytes: AlignedBytes,
    _phantom: PhantomData<[T]>,
}

impl<T: Copy> Array<T> {
    pub fn len(&self) -> usize {
        self.bytes.len() / std::mem::size_of::<T>()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn as_ptr(&self) -> *const T {
        self.bytes.as_ptr() as *const T
    }
}

impl<T: Copy> std::ops::Index<usize> for Array<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        // SAFETY: the bytes are guaranteed to hold valid T values
        unsafe {
            let ptr = self.as_ptr().add(index);
            &*ptr
        }
    }
}

impl<T: Copy> std::ops::IndexMut<usize> for Array<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        // SAFETY: the bytes are guaranteed to hold valid T values
        unsafe {
            let ptr = self.as_ptr().add(index).cast_mut();
            &mut *ptr
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::util::reflection::{ConstTypeName, TypeInfo, TypeInfoOptions};

    use super::*;

    // provide these impls for testing purposes only
    static U32_TYPE_INFO: TypeInfo = TypeInfo::new::<u32>(TypeInfoOptions {
        is_zeroable: true,
        mem_repr: Some(&[(0, lir::MemOpType::I32)]),
    });
    unsafe impl Reflect for u32 {
        const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&U32_TYPE_INFO);
    }
    static STRING_TYPE_INFO: TypeInfo =
        TypeInfo::new::<String>(TypeInfoOptions { is_zeroable: false, mem_repr: None });
    unsafe impl Reflect for String {
        const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&STRING_TYPE_INFO);
    }
    impl ConstTypeName for String {
        const TYPE_NAME: &'static str = "String";
    }
    static F64_TYPE_INFO: TypeInfo = TypeInfo::new::<f64>(TypeInfoOptions {
        is_zeroable: true,
        mem_repr: Some(&[(0, lir::MemOpType::F64)]),
    });
    unsafe impl Reflect for f64 {
        const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&F64_TYPE_INFO);
    }
    impl ConstTypeName for f64 {
        const TYPE_NAME: &'static str = "f64";
    }
    static BOOL_TYPE_INFO: TypeInfo = TypeInfo::new::<bool>(TypeInfoOptions {
        is_zeroable: true,
        mem_repr: Some(&[(0, lir::MemOpType::I8)]),
    });
    unsafe impl Reflect for bool {
        const CONCRETE_TY: ConcreteTy = ConcreteTy::new(&BOOL_TYPE_INFO);
    }
    impl ConstTypeName for bool {
        const TYPE_NAME: &'static str = "bool";
    }

    #[test]
    fn test_basic_insert_retrieve() {
        // Schema with a single u32 field
        let schema = RowSchema::new(&[u32::CONCRETE_TY], true);
        let mut buffer = RowBuffer::new(schema);
        buffer.ensure_capacity(1);
        {
            let mut row = buffer.row_mut(0);
            row.insert::<u32>(0, 42);
        }
        let row = buffer.row(0);
        assert_eq!(row.get::<u32>(0), Some(&42));
    }

    #[test]
    fn test_heterogeneous_fields() {
        let schema =
            RowSchema::new(&[u32::CONCRETE_TY, String::CONCRETE_TY, f64::CONCRETE_TY], true);
        let mut buffer = RowBuffer::new(schema);
        buffer.ensure_capacity(1);

        {
            let mut row = buffer.row_mut(0);
            row.insert::<u32>(0, 42);
            row.insert::<String>(1, "hello".to_string());
            row.insert::<f64>(2, 3.14);
        }

        let row = buffer.row(0);
        assert_eq!(row.get::<u32>(0), Some(&42));
        assert_eq!(row.get::<String>(1), Some(&"hello".to_string()));
        assert_eq!(row.get::<f64>(2), Some(&3.14));
    }

    #[test]
    fn test_sparse_fields() {
        // create a schema with 4 fields but only insert 2 of them
        let schema = RowSchema::new(
            &[u32::CONCRETE_TY, String::CONCRETE_TY, f64::CONCRETE_TY, bool::CONCRETE_TY],
            true,
        );
        let mut buffer = RowBuffer::new(schema);
        buffer.ensure_capacity(1);

        {
            let mut row = buffer.row_mut(0);
            // only insert fields 0 and 2, leave 1 and 3 empty
            row.insert::<u32>(0, 100);
            row.insert::<f64>(2, 2.718);
        }

        let row = buffer.row(0);

        // check that inserted fields are present
        assert_eq!(row.get::<u32>(0), Some(&100));
        assert_eq!(row.get::<f64>(2), Some(&2.718));

        // check that non-inserted fields are not present
        assert_eq!(row.get::<String>(1), None);
        assert_eq!(row.get::<bool>(3), None);

        // check that has_field works correctly
        assert!(row.has_field(0));
        assert!(!row.has_field(1));
        assert!(row.has_field(2));
        assert!(!row.has_field(3));
    }

    #[test]
    fn test_insert_and_take() {
        let schema =
            RowSchema::new(&[u32::CONCRETE_TY, String::CONCRETE_TY, f64::CONCRETE_TY], true);
        let mut buffer = RowBuffer::new(schema);
        buffer.ensure_capacity(1);

        {
            let mut row = buffer.row_mut(0);

            // insert fields
            row.insert::<u32>(0, 42);
            row.insert::<String>(1, "hello".to_string());
            row.insert::<f64>(2, 3.14);

            // verify they are present
            assert!(row.has_field(0));
            assert!(row.has_field(1));
            assert!(row.has_field(2));

            // take the fields
            let value1 = row.take::<u32>(0);
            let value2 = row.take::<String>(1);
            let value3 = row.take::<f64>(2);

            // verify the taken values are correct
            assert_eq!(value1, Some(42));
            assert_eq!(value2, Some("hello".to_string()));
            assert_eq!(value3, Some(3.14));

            // verify fields are now absent
            assert!(!row.has_field(0));
            assert!(!row.has_field(1));
            assert!(!row.has_field(2));

            // verify get returns None for taken fields
            assert_eq!(row.get::<u32>(0), None);
            assert_eq!(row.get::<String>(1), None);
            assert_eq!(row.get::<f64>(2), None);
        }
    }

    #[test]
    #[should_panic(expected = "type mismatch")]
    fn test_type_mismatch_present_field() {
        let schema = RowSchema::new(&[u32::CONCRETE_TY], true);
        let mut buffer = RowBuffer::new(schema);
        buffer.ensure_capacity(1);

        {
            let mut row = buffer.row_mut(0);
            row.insert::<u32>(0, 42);
        }

        let row = buffer.row(0);
        // Try to access a u32 field as a String - should panic
        let _: Option<&String> = row.get::<String>(0);
    }

    #[test]
    #[should_panic(expected = "type mismatch")]
    fn test_type_mismatch_absent_field() {
        let schema = RowSchema::new(&[u32::CONCRETE_TY], true);
        let mut buffer = RowBuffer::new(schema);
        buffer.ensure_capacity(1);

        let row = buffer.row(0);
        // Try to access a u32 field as a String when field is not present - should still panic
        let _: Option<&String> = row.get::<String>(0);
    }

    #[test]
    fn test_capacity_expansion_preserves_data() {
        let schema =
            RowSchema::new(&[u32::CONCRETE_TY, String::CONCRETE_TY, f64::CONCRETE_TY], true);
        let mut buffer = RowBuffer::new(schema);

        // insert data into the first few rows
        buffer.ensure_capacity(3);
        {
            let mut row0 = buffer.row_mut(0);
            row0.insert::<u32>(0, 100);
            row0.insert::<String>(1, "first".to_string());
            row0.insert::<f64>(2, 1.0);
        }
        {
            let mut row1 = buffer.row_mut(1);
            row1.insert::<u32>(0, 200);
            row1.insert::<f64>(2, 2.0);
            // leave field 1 empty
        }
        {
            let mut row2 = buffer.row_mut(2);
            row2.insert::<String>(1, "third".to_string());
            // leave fields 0 and 2 empty
        }

        // expand capacity beyond initial 100 rows
        buffer.ensure_capacity(150);

        // verify that all data is still present and correct
        let row0 = buffer.row(0);
        assert_eq!(row0.get::<u32>(0), Some(&100));
        assert_eq!(row0.get::<String>(1), Some(&"first".to_string()));
        assert_eq!(row0.get::<f64>(2), Some(&1.0));

        let row1 = buffer.row(1);
        assert_eq!(row1.get::<u32>(0), Some(&200));
        assert_eq!(row1.get::<String>(1), None);
        assert_eq!(row1.get::<f64>(2), Some(&2.0));

        let row2 = buffer.row(2);
        assert_eq!(row2.get::<u32>(0), None);
        assert_eq!(row2.get::<String>(1), Some(&"third".to_string()));
        assert_eq!(row2.get::<f64>(2), None);

        // verify that we can still insert into new rows after expansion
        {
            let mut row100 = buffer.row_mut(100);
            row100.insert::<u32>(0, 1000);
            row100.insert::<String>(1, "hundredth".to_string());
        }

        let row100 = buffer.row(100);
        assert_eq!(row100.get::<u32>(0), Some(&1000));
        assert_eq!(row100.get::<String>(1), Some(&"hundredth".to_string()));
        assert_eq!(row100.get::<f64>(2), None);
    }

    #[test]
    fn test_massive_row_count() {
        let schema = RowSchema::new(
            &[
                bool::CONCRETE_TY,
                bool::CONCRETE_TY,
                u32::CONCRETE_TY,
                f64::CONCRETE_TY,
                String::CONCRETE_TY,
            ],
            true,
        );
        let mut buffer = RowBuffer::new(schema);

        // create 200 rows with predictable values
        let num_rows = 200;
        buffer.ensure_capacity(num_rows);

        // populate all rows with predictable values
        for row_idx in 0..num_rows {
            let mut row = buffer.row_mut(row_idx);

            row.insert::<bool>(0, (row_idx & 1) != 0);
            row.insert::<bool>(1, (row_idx & 2) != 0);
            row.insert::<u32>(2, row_idx as u32);
            row.insert::<f64>(3, row_idx as f64);
            row.insert::<String>(4, row_idx.to_string());
        }

        // verify all rows have correct data
        for row_idx in 0..num_rows {
            let row = buffer.row(row_idx);

            assert_eq!(row.get::<bool>(0), Some(&((row_idx & 1) != 0)));
            assert_eq!(row.get::<bool>(1), Some(&((row_idx & 2) != 0)));
            assert_eq!(row.get::<u32>(2), Some(&(row_idx as u32)));
            assert_eq!(row.get::<f64>(3), Some(&(row_idx as f64)));
            assert_eq!(row.get::<String>(4), Some(&row_idx.to_string()));

            // verify all fields are present
            assert!(row.has_field(0));
            assert!(row.has_field(1));
            assert!(row.has_field(2));
            assert!(row.has_field(3));
            assert!(row.has_field(4));
        }
    }

    #[test]
    fn test_change_schema() {
        // create initial buffer with (u32, String) schema
        let old_schema = RowSchema::new(&[u32::CONCRETE_TY, String::CONCRETE_TY], true);
        let mut buffer = RowBuffer::new_with_capacity(old_schema, 3);

        // populate with some data
        {
            let mut row0 = buffer.row_mut(0);
            row0.insert::<u32>(0, 42);
            row0.insert::<String>(1, "hello".to_string());
        }
        {
            let mut row1 = buffer.row_mut(1);
            row1.insert::<u32>(0, 100);
            row1.insert::<String>(1, "world".to_string());
        }
        {
            let mut row2 = buffer.row_mut(2);
            row2.insert::<u32>(0, 999);
            row2.insert::<String>(1, "test".to_string());
        }

        // create new schema with (String, u32) - reversed order
        let new_schema = RowSchema::new(&[String::CONCRETE_TY, u32::CONCRETE_TY], true);

        // change schema, taking ownership of values
        buffer.change_schema(new_schema, |mut old_row, mut new_row| {
            // take the old values and insert them in new positions
            new_row.insert::<u32>(1, old_row.take::<u32>(0).unwrap());
            new_row.insert::<String>(0, old_row.take::<String>(1).unwrap());
        });

        // verify the data is now in the new schema
        let row0 = buffer.row(0);
        assert_eq!(row0.get::<String>(0), Some(&"hello".to_string()));
        assert_eq!(row0.get::<u32>(1), Some(&42));

        let row1 = buffer.row(1);
        assert_eq!(row1.get::<String>(0), Some(&"world".to_string()));
        assert_eq!(row1.get::<u32>(1), Some(&100));

        let row2 = buffer.row(2);
        assert_eq!(row2.get::<String>(0), Some(&"test".to_string()));
        assert_eq!(row2.get::<u32>(1), Some(&999));
    }

    #[test]
    fn test_insert_zeroable() {
        let schema = RowSchema::new(&[u32::CONCRETE_TY], true);
        let mut buffer = RowBuffer::new(schema);
        buffer.ensure_capacity(1);

        {
            let mut row = buffer.row_mut(0);

            // insert a nonzero value
            row.insert::<u32>(0, 42);
            assert_eq!(row.get::<u32>(0), Some(&42));

            // take the value out
            let taken = row.take::<u32>(0);
            assert_eq!(taken, Some(42));
            assert_eq!(row.get::<u32>(0), None);

            // insert zeroable
            row.insert_zeroable(0);
            assert_eq!(row.get::<u32>(0), Some(&0));
        }
    }

    #[test]
    fn test_clear() {
        let schema =
            RowSchema::new(&[u32::CONCRETE_TY, String::CONCRETE_TY, f64::CONCRETE_TY], true);
        let mut buffer = RowBuffer::new_with_capacity(schema, 3);

        // populate multiple rows with data
        {
            let mut row0 = buffer.row_mut(0);
            row0.insert::<u32>(0, 100);
            row0.insert::<String>(1, "first".to_string());
            row0.insert::<f64>(2, 1.0);
        }
        {
            let mut row1 = buffer.row_mut(1);
            row1.insert::<u32>(0, 200);
            row1.insert::<f64>(2, 2.0);
            // leave field 1 empty
        }
        {
            let mut row2 = buffer.row_mut(2);
            row2.insert::<String>(1, "third".to_string());
            // leave fields 0 and 2 empty
        }

        // verify data is present before clearing
        let row0 = buffer.row(0);
        assert!(row0.has_field(0));
        assert!(row0.has_field(1));
        assert!(row0.has_field(2));

        let row1 = buffer.row(1);
        assert!(row1.has_field(0));
        assert!(!row1.has_field(1));
        assert!(row1.has_field(2));

        let row2 = buffer.row(2);
        assert!(!row2.has_field(0));
        assert!(row2.has_field(1));
        assert!(!row2.has_field(2));

        // clear the buffer
        buffer.clear();

        // verify all fields are now absent
        let row0 = buffer.row(0);
        assert!(!row0.has_field(0));
        assert!(!row0.has_field(1));
        assert!(!row0.has_field(2));
        assert_eq!(row0.get::<u32>(0), None);
        assert_eq!(row0.get::<String>(1), None);
        assert_eq!(row0.get::<f64>(2), None);

        let row1 = buffer.row(1);
        assert!(!row1.has_field(0));
        assert!(!row1.has_field(1));
        assert!(!row1.has_field(2));

        let row2 = buffer.row(2);
        assert!(!row2.has_field(0));
        assert!(!row2.has_field(1));
        assert!(!row2.has_field(2));
    }

    // === Always-present (occupancy_bitfield = false) tests ===

    #[test]
    fn test_basic_always_present() {
        let schema = RowSchema::new(&[u32::CONCRETE_TY], false);
        let mut buffer = RowBuffer::new(schema);
        buffer.ensure_capacity(1);
        {
            let mut row = buffer.row_mut(0);
            let x = row.get_mut::<u32>(0).unwrap();
            *x = 42;
        }
        let row = buffer.row(0);
        assert_eq!(row.get::<u32>(0), Some(&42));
    }

    #[test]
    fn test_heterogeneous_always_present() {
        let schema =
            RowSchema::new(&[u32::CONCRETE_TY, bool::CONCRETE_TY, f64::CONCRETE_TY], false);
        let mut buffer = RowBuffer::new(schema);
        buffer.ensure_capacity(1);
        {
            let mut row = buffer.row_mut(0);
            *row.get_mut::<u32>(0).unwrap() = 42;
            *row.get_mut::<bool>(1).unwrap() = true;
            *row.get_mut::<f64>(2).unwrap() = 3.14;
        }
        let row = buffer.row(0);
        assert_eq!(row.get::<u32>(0), Some(&42));
        assert_eq!(row.get::<bool>(1), Some(&true));
        assert_eq!(row.get::<f64>(2), Some(&3.14));
    }

    #[test]
    fn test_massive_always_present() {
        let schema = RowSchema::new(
            &[bool::CONCRETE_TY, bool::CONCRETE_TY, u32::CONCRETE_TY, f64::CONCRETE_TY],
            false,
        );
        let mut buffer = RowBuffer::new(schema);
        let num_rows = 200;
        buffer.ensure_capacity(num_rows);
        for row_idx in 0..num_rows {
            let mut row = buffer.row_mut(row_idx);
            *row.get_mut::<bool>(0).unwrap() = (row_idx & 1) != 0;
            *row.get_mut::<bool>(1).unwrap() = (row_idx & 2) != 0;
            *row.get_mut::<u32>(2).unwrap() = row_idx as u32;
            *row.get_mut::<f64>(3).unwrap() = row_idx as f64;
        }
        for row_idx in 0..num_rows {
            let row = buffer.row(row_idx);
            assert_eq!(row.get::<bool>(0), Some(&((row_idx & 1) != 0)));
            assert_eq!(row.get::<bool>(1), Some(&((row_idx & 2) != 0)));
            assert_eq!(row.get::<u32>(2), Some(&(row_idx as u32)));
            assert_eq!(row.get::<f64>(3), Some(&(row_idx as f64)));
        }
    }

    #[test]
    fn test_insert_zeroable_always_present() {
        let schema = RowSchema::new(&[u32::CONCRETE_TY], false);
        let mut buffer = RowBuffer::new(schema);
        buffer.ensure_capacity(1);
        {
            let mut row = buffer.row_mut(0);
            *row.get_mut::<u32>(0).unwrap() = 42;
            *row.get_mut::<u32>(0).unwrap() = 0;
        }
        let row = buffer.row(0);
        assert_eq!(row.get::<u32>(0), Some(&0));
    }

    #[test]
    #[should_panic(
        expected = "cannot initialize an always-present row buffer when field at index 0 is not zeroable"
    )]
    fn test_panic_nonzeroable_always_present() {
        // String is not zeroable
        let schema = RowSchema::new(&[String::CONCRETE_TY], false);
        let _ = RowBuffer::new(schema);
    }

    #[test]
    #[should_panic(expected = "cannot take a field if the occupancy bitfield is omitted")]
    fn test_panic_take_always_present() {
        let schema = RowSchema::new(&[u32::CONCRETE_TY], false);
        let mut buffer = RowBuffer::new(schema);
        buffer.ensure_capacity(1);
        let mut row = buffer.row_mut(0);
        let _ = row.take::<u32>(0);
    }

    #[test]
    fn test_take_array() {
        // create a schema with a single u32 field, always present
        let schema = RowSchema::new(&[u32::CONCRETE_TY], false);
        let mut buffer = RowBuffer::new_with_capacity(schema, 5);

        // populate the buffer with some values
        for i in 0..5 {
            let mut row = buffer.row_mut(i);
            *row.get_mut::<u32>(0).unwrap() = i as u32;
        }

        // verify the data is there before taking
        for i in 0..5 {
            let row = buffer.row(i);
            assert_eq!(row.get::<u32>(0), Some(&(i as u32)));
        }

        // take the array
        let array = buffer.take_array::<u32>();

        // verify the array has the correct data
        assert_eq!(array.len(), 5);
        for i in 0..5 {
            assert_eq!(array[i], i as u32);
        }

        // verify the buffer is now zeroed out
        for i in 0..5 {
            let row = buffer.row(i);
            assert_eq!(row.get::<u32>(0), Some(&0));
        }
    }

    #[test]
    #[should_panic(expected = "cannot reinterpret the row buffer as an array of type u32")]
    fn test_take_array_wrong_type() {
        // create a schema with a single f64 field, always present
        let schema = RowSchema::new(&[f64::CONCRETE_TY], false);
        let mut buffer = RowBuffer::new_with_capacity(schema, 3);

        // try to take as u32 - should panic
        let _ = buffer.take_array::<u32>();
    }

    #[test]
    #[should_panic(expected = "cannot reinterpret the row buffer as an array of type u32")]
    fn test_take_array_multiple_fields() {
        // create a schema with multiple fields, always present
        let schema = RowSchema::new(&[u32::CONCRETE_TY, f64::CONCRETE_TY], false);
        let mut buffer = RowBuffer::new_with_capacity(schema, 3);

        // try to take as u32 - should panic because there are multiple fields
        let _ = buffer.take_array::<u32>();
    }

    #[test]
    #[should_panic(expected = "cannot reinterpret the row buffer as an array of type u32")]
    fn test_take_array_with_occupancy_bitfield() {
        // create a schema with occupancy bitfield (sparse fields)
        let schema = RowSchema::new(&[u32::CONCRETE_TY], true);
        let mut buffer = RowBuffer::new_with_capacity(schema, 3);

        // try to take as u32 - should panic because there's an occupancy bitfield
        let _ = buffer.take_array::<u32>();
    }
}
