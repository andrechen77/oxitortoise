#[derive(Debug)]
pub enum Value {
    Float(Float),
    // TODO other types: string, boolean, agent, agentset, list, reporter, command
}

impl TryFrom<&Value> for u64 {
    type Error = ();
    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            &Value::Float(Float(f)) if f.is_finite() && f >= 0.0 && f < u64::MAX as f64 => {
                Ok(f as u64)
            }
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct Float(pub f64);
