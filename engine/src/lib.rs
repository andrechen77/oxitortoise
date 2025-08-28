// for the row buffer
#![feature(slice_ptr_get, alloc_layout_extra, const_type_name, fn_traits, unboxed_closures)]

pub extern crate flagset;
pub extern crate lir;
pub extern crate slotmap;

pub mod exec;
pub mod mir;
pub mod sim;
pub mod updater;
pub mod util;
pub mod workspace;
