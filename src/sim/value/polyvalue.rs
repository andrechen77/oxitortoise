//! A container that can hold any of the possible types of values in NetLogo.
//!
//! The inner value of a `PolyValue` can be safely retrieved using using the
//! [`PolyValue::get`] and [`PolyValue::get_mut`] methods, or unsafely using the
//! [`PolyValue::get_unchecked`] and [`PolyValue::get_mut_unchecked`] methods.
//! The inner value is always safely dropped when the `PolyValue` is dropped.
//!

use derive_more::derive::{From, TryInto};

use crate::sim::{patch::PatchId, turtle::TurtleId};

use super::{Boolean, Float, String};

#[derive(Debug, Clone, From, TryInto, PartialEq)]
#[try_into(owned, ref, ref_mut)]
#[non_exhaustive]
#[repr(C, u8)]
pub enum PolyValue {
    /// Indicates that there is no data stored in the value. Dropping a
    /// [`PolyValue`] with this type does nothing.
    Uninit = 0,
    Float(Float),
    Boolean(Boolean),
    String(String),
    Nobody,
    Turtle(TurtleId),
    Patch(PatchId),
}

impl Default for PolyValue {
    fn default() -> Self {
        PolyValue::Float(Float::new(0.0))
    }
}

impl PolyValue {
    pub fn get<T>(&self) -> Option<&T>
    where
        for<'a> &'a Self: TryInto<&'a T>,
    {
        self.try_into().ok()
    }

    pub fn get_mut<T>(&mut self) -> Option<&mut T>
    where
        for<'a> &'a mut Self: TryInto<&'a mut T>,
    {
        self.try_into().ok()
    }

    pub fn into<T>(self) -> Option<T>
    where
        Self: TryInto<T>,
    {
        self.try_into().ok()
    }
}
