use std::cell::Cell;

use crate::sim::value;

/// A sentinel value representing a clear tick counter.
const CLEAR_TICK: f64 = -1.0;

/// The current tick number of an NetLogo simulation instance.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Tick(Cell<f64>);

impl Default for Tick {
    fn default() -> Self {
        Self(Cell::new(CLEAR_TICK))
    }
}

impl Tick {
    pub fn get(&self) -> Option<value::Float> {
        if self.is_clear() {
            None
        } else {
            Some(value::Float::new(self.0.get()))
        }
    }

    /// Attempts to advance the tick counter by one. Returns `true` if successful, and
    /// false if the counter was cleared.
    pub fn advance(&self) -> Result<(), ()> {
        self.advance_by(1.0)
    }

    /// Attempts to advance the tick counter by the specified amount. Returns `true` if
    /// successful, and false if the counter was cleared.
    pub fn advance_by(&self, amount: f64) -> Result<(), ()> {
        if self.is_clear() {
            return Err(());
        }

        self.0.set(self.0.get() + amount);
        Ok(())
    }

    pub fn is_clear(&self) -> bool {
        self.0.get() == CLEAR_TICK
    }

    /// Clears the tick counter.
    pub fn clear(&self) {
        self.0.set(CLEAR_TICK);
    }

    /// Sets the tick counter to zero.
    pub fn reset(&self) {
        self.0.set(0.0);
    }
}

impl TryFrom<Tick> for value::Float {
    type Error = ();

    /// Converts a tick number into a NetLogo float value. Errors if the tick
    /// counter was clear
    fn try_from(tick: Tick) -> Result<Self, Self::Error> {
        tick.get().ok_or(())
    }
}
