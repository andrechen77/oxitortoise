use crate::sim::value;

/// The current tick number of an NetLogo simulation instance. The tick counter
/// can be a zero/positive value, or NaN (representing a tick counter that
/// hasn't started).
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Tick(f64);

impl Default for Tick {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct TickAdvanceError;

impl Tick {
    pub fn new() -> Self {
        Self(f64::NAN)
    }

    /// Returns the current tick number as a NetLogo float value. Returns `None`
    /// if the tick counter is in a clear state.
    pub fn get(&self) -> Option<value::NlFloat> {
        if self.is_clear() { None } else { Some(value::NlFloat::new(self.0)) }
    }

    /// Attempts to advance the tick counter by one. Errors if the counter is
    /// in a clear state.
    pub fn advance(&mut self) -> Result<(), TickAdvanceError> {
        self.advance_by(1.0)
    }

    /// Attempts to advance the tick counter by the specified amount. Returns `true` if
    /// successful, and false if the counter was cleared.
    pub fn advance_by(&mut self, amount: f64) -> Result<(), TickAdvanceError> {
        if self.is_clear() {
            return Err(TickAdvanceError);
        }

        self.0 += amount;
        Ok(())
    }

    pub fn is_clear(&self) -> bool {
        self.0.is_nan()
    }

    /// Clears the tick counter.
    pub fn clear(&mut self) {
        self.0 = f64::NAN;
    }

    /// Sets the tick counter to zero.
    pub fn reset(&mut self) {
        self.0 = 0.0;
    }
}

impl TryFrom<Tick> for value::NlFloat {
    type Error = ();

    /// Converts a tick number into a NetLogo float value. Errors if the tick
    /// counter was clear
    fn try_from(tick: Tick) -> Result<Self, Self::Error> {
        tick.get().ok_or(())
    }
}
