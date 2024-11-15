use derive_more::derive::From;

use crate::topology::CoordInt;

#[derive(Debug, Clone, From, PartialEq)]
pub enum Value {
    #[from]
    Float(Float),
    // TODO other types: string, boolean, agent, agentset, list, reporter, command
}

impl Default for Value {
    fn default() -> Self {
        Value::Float(Float(0.0))
    }
}

impl Value {
    pub fn to_u64_round_to_zero(&self) -> Option<u64> {
        match self {
            &Value::Float(Float(f)) if f.is_finite() && f >= 0.0 && f < u64::MAX as f64 => {
                Some(f as u64)
            }
            _ => None,
        }
    }

    pub fn to_f64(&self) -> Option<f64> {
        match self {
            &Value::Float(Float(f)) if f.is_finite() => Some(f),
            _ => None,
        }
    }

    /// Resets the value to its default value.
    /// TODO should this always be zero?
    pub fn reset(&mut self) {
        *self = Value::default();
    }
}

#[derive(Debug, Clone, From, PartialEq, PartialOrd)] // TODO also ord and eq
pub struct Float(pub f64);

impl From<CoordInt> for Float {
    fn from(value: CoordInt) -> Self {
        Float(value as f64)
    }
}

#[derive(Debug, Clone)]
pub struct String(/* TODO */);

impl From<&str> for String {
    fn from(_value: &str) -> Self {
        todo!()
    }
}
