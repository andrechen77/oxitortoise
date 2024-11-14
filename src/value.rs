#[derive(Debug)]
pub enum Value {
    Float(Float),
    // TODO other types: string, boolean, agent, agentset, list, reporter, command
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
}

#[derive(Debug)]
pub struct Float(pub f64);

#[derive(Debug)]
pub struct String(/* TODO */);

impl From<&str> for String {
    fn from(value: &str) -> Self {
        todo!()
    }
}
