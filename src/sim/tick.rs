use std::cell::Cell;

use crate::sim::value;

/// A sentinel value representing a clear tick counter.
const CLEAR_TICK: f64 = -1.0;

/// The current tick number of an NetLogo simulation instance.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Tick(Cell<f64>);

impl Default for Tick {
    fn default() -> Self {
        Self(Cell::new(CLEAR_TICK))
    }
}

impl Tick {
    pub fn get(&self) -> Option<f64> {
        if self.is_clear() {
            None
        } else {
            Some(self.0.get())
        }
    }

    /// Attempts to advance the tick counter by one. Returns `true` if successful, and
    /// false if the counter was cleared.
    #[must_use]
    pub fn advance(&self) -> bool {
        self.advance_by(1.0)
    }

    /// Attempts to advance the tick counter by the specified amount. Returns `true` if
    /// successful, and false if the counter was cleared.
    #[must_use]
    pub fn advance_by(&self, amount: f64) -> bool {
        if self.is_clear() {
            return false;
        }

        self.0.set(self.0.get() + amount);
        true
    }

    pub fn is_clear(&self) -> bool {
        self.0.get() == CLEAR_TICK
    }

    pub fn clear(&self) {
        self.0.set(CLEAR_TICK);
    }
}

impl TryFrom<Tick> for value::Float {
    type Error = ();

    /// Converts a tick number into a NetLogo float value. Errors if the tick
    /// counter was clear
    fn try_from(tick: Tick) -> Result<Self, Self::Error> {
        if tick.is_clear() {
            Err(())
        } else {
            Ok(value::Float::new(tick.0.get()))
        }
    }
}
