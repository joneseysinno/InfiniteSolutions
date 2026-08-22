//! [`Instant`] — *now*, handed in from outside.

/// A monotonic instant, in nanoseconds since an unspecified origin.
///
/// The runtime owns time (D5) but owns no thread (L1), so it does not *read* a clock —
/// `now` arrives as an argument to `tick`, supplied by the `Clock` port.
///
/// Deliberately not `std::time::Instant`. A wall clock makes the schedule
/// irreproducible, and D19's equivalence law — a compiled block must be
/// *observationally identical* to the interpreted one — is only checkable if the
/// schedule is deterministic. A plain counter makes every test replayable.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(u64);

impl Instant {
    /// The origin.
    pub const ZERO: Instant = Instant(0);

    /// An instant at `nanos` since the origin.
    pub const fn from_nanos(nanos: u64) -> Self {
        Self(nanos)
    }

    /// Nanoseconds since the origin.
    pub const fn as_nanos(self) -> u64 {
        self.0
    }

    /// Nanoseconds elapsed since `earlier`, saturating at zero.
    ///
    /// Saturating rather than panicking: a non-monotonic `Clock` is a port defect and
    /// must not be able to bring down a tick.
    pub const fn saturating_since(self, earlier: Instant) -> u64 {
        self.0.saturating_sub(earlier.0)
    }
}
