use std::fmt;

use super::TryAsFloat;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct String(Box<()>);
// for now, any non-copy pointer-sized type will do, to simulate what the
// actual string type would look like

impl String {
    pub fn new() -> Self {
        Self(Box::new(()))
    }
}

impl From<&str> for String {
    fn from(_value: &str) -> Self {
        todo!()
    }
}

impl Default for String {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "STRINGDISPLAYISNOTIMPLEMENTED") // TODO do this
    }
}

impl TryAsFloat for String {
    fn try_as_float(&self) -> Option<crate::sim::value::Float> {
        None
    }
}
