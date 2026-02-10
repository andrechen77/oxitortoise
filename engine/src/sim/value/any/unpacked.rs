use crate::{
    sim::{
        patch::PatchId,
        turtle::TurtleId,
        value::{NlBool, NlFloat},
    },
    util::reflection::{ConcreteTy, Reflect as _},
};
use std::{
    cmp::Ordering,
    ops::{Add, Div, Mul, Sub},
};

use super::{BoxedAny, PackedAny};

#[derive(Debug)]
pub enum UnpackedAny {
    Float(f64),
    Bool(bool),
    Nobody,
    Turtle(TurtleId),
    Patch(PatchId),
    Link(LinkId),
    Other(BoxedAny),
}

impl UnpackedAny {
    pub fn ty(&self) -> ConcreteTy {
        match self {
            UnpackedAny::Bool(_) => NlBool::CONCRETE_TY,
            UnpackedAny::Float(_) => NlFloat::CONCRETE_TY,
            UnpackedAny::Nobody => NlBool::CONCRETE_TY,
            UnpackedAny::Turtle(_) => TurtleId::CONCRETE_TY,
            UnpackedAny::Patch(_) => PatchId::CONCRETE_TY,
            UnpackedAny::Link(_) => todo!("add link id"),
            UnpackedAny::Other(_) => todo!("match on the inner type"),
        }
    }

    pub fn and(self, rhs: Self) -> bool {
        match (self, rhs) {
            (UnpackedAny::Bool(lhs), UnpackedAny::Bool(rhs)) => lhs && rhs,
            (lhs, rhs) => unimplemented!("Anding {:?} and {:?} is not supported", lhs, rhs),
        }
    }

    pub fn or(self, rhs: Self) -> bool {
        match (self, rhs) {
            (UnpackedAny::Bool(lhs), UnpackedAny::Bool(rhs)) => lhs || rhs,
            (lhs, rhs) => unimplemented!("Oring {:?} and {:?} is not supported", lhs, rhs),
        }
    }
}

impl Clone for UnpackedAny {
    fn clone(&self) -> Self {
        match self {
            UnpackedAny::Float(value) => UnpackedAny::Float(*value),
            UnpackedAny::Bool(value) => UnpackedAny::Bool(*value),
            UnpackedAny::Turtle(value) => UnpackedAny::Turtle(*value),
            _ => unimplemented!(),
        }
    }
}

impl Add for UnpackedAny {
    type Output = UnpackedAny;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (UnpackedAny::Float(lhs), UnpackedAny::Float(rhs)) => UnpackedAny::Float(lhs + rhs),
            (lhs, rhs) => unimplemented!("Adding {:?} and {:?} is not supported", lhs, rhs),
        }
    }
}

impl Sub for UnpackedAny {
    type Output = UnpackedAny;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (UnpackedAny::Float(lhs), UnpackedAny::Float(rhs)) => UnpackedAny::Float(lhs - rhs),
            (lhs, rhs) => unimplemented!("Subtracting {:?} and {:?} is not supported", lhs, rhs),
        }
    }
}

impl Mul for UnpackedAny {
    type Output = UnpackedAny;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (UnpackedAny::Float(lhs), UnpackedAny::Float(rhs)) => UnpackedAny::Float(lhs * rhs),
            (lhs, rhs) => unimplemented!("Multiplying {:?} and {:?} is not supported", lhs, rhs),
        }
    }
}

impl Div for UnpackedAny {
    type Output = UnpackedAny;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (UnpackedAny::Float(lhs), UnpackedAny::Float(rhs)) => UnpackedAny::Float(lhs / rhs),
            (lhs, rhs) => unimplemented!("Dividing {:?} and {:?} is not supported", lhs, rhs),
        }
    }
}

impl PartialEq for UnpackedAny {
    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (UnpackedAny::Float(lhs), UnpackedAny::Float(rhs)) => lhs == rhs,
            (lhs, rhs) => unimplemented!("Comparing {:?} and {:?} is not supported", lhs, rhs),
        }
    }
}

impl PartialOrd for UnpackedAny {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        match (self, rhs) {
            (UnpackedAny::Float(lhs), UnpackedAny::Float(rhs)) => lhs.partial_cmp(rhs),
            (lhs, rhs) => unimplemented!("Comparing {:?} and {:?} is not supported", lhs, rhs),
        }
    }
}

impl Add for PackedAny {
    type Output = PackedAny;
    fn add(self, rhs: Self) -> Self::Output {
        PackedAny::pack(self.unpack() + rhs.unpack())
    }
}

impl Sub for PackedAny {
    type Output = PackedAny;
    fn sub(self, rhs: Self) -> Self::Output {
        PackedAny::pack(self.unpack() - rhs.unpack())
    }
}

impl Mul for PackedAny {
    type Output = PackedAny;
    fn mul(self, rhs: Self) -> Self::Output {
        PackedAny::pack(self.unpack() * rhs.unpack())
    }
}

impl Div for PackedAny {
    type Output = PackedAny;
    fn div(self, rhs: Self) -> Self::Output {
        PackedAny::pack(self.unpack() / rhs.unpack())
    }
}

impl PartialEq for PackedAny {
    fn eq(&self, rhs: &Self) -> bool {
        self.unpack() == rhs.unpack()
    }
}

impl PartialOrd for PackedAny {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        self.unpack().partial_cmp(&rhs.unpack())
    }
}

// placeholder; remove once links are implemented
#[derive(Debug)]
pub struct LinkId;
