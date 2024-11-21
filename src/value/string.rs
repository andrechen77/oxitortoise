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
