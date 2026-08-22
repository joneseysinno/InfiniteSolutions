//! [`Budget`] — how much a tick is allowed to do.

use super::Instant;

/// The work a single `tick` may spend.
///
/// Two independent limits, both required. A unit count alone cannot protect a frame
/// when one unit turns out to be expensive; a deadline alone cannot make a schedule
/// reproducible, because it depends on how fast the machine is. Together, the unit
/// count is what tests assert against and the deadline is what protects the frame
/// (spec §7.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Budget {
    units: u32,
    deadline: Option<Instant>,
}

impl Budget {
    /// A budget of `units` work units and no deadline. The deterministic form; this is
    /// what tests use.
    pub const fn units(units: u32) -> Self {
        Self {
            units,
            deadline: None,
        }
    }

    /// Adds a wall deadline. Reaching it exhausts the budget regardless of units left.
    pub const fn until(mut self, deadline: Instant) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Work units remaining.
    pub const fn remaining(&self) -> u32 {
        self.units
    }

    /// Whether the budget is spent as of `now`.
    pub fn is_exhausted(&self, now: Instant) -> bool {
        if self.units == 0 {
            return true;
        }
        match self.deadline {
            Some(deadline) => now >= deadline,
            None => false,
        }
    }

    /// Spends one work unit. Saturates at zero rather than wrapping.
    pub fn spend(&mut self) {
        self.units = self.units.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{Budget, Instant};

    #[test]
    fn units_exhaust() {
        let mut b = Budget::units(2);
        assert!(!b.is_exhausted(Instant::ZERO));
        b.spend();
        b.spend();
        assert!(b.is_exhausted(Instant::ZERO));
    }

    #[test]
    fn deadline_exhausts_before_units_run_out() {
        let b = Budget::units(100).until(Instant::from_nanos(50));
        assert!(!b.is_exhausted(Instant::from_nanos(49)));
        assert!(b.is_exhausted(Instant::from_nanos(50)));
        assert_eq!(b.remaining(), 100);
    }
}
