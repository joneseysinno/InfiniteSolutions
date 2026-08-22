//! [`Clock`] — where *now* comes from.

use crate::core::Instant;

/// A monotonic instant source.
///
/// The runtime owns time (D5) but owns no thread (L1), so it does not read a clock on
/// its own schedule — the caller reads one and hands the result to `tick`. This trait
/// exists so that a test can supply a counter and replay a schedule exactly, which is
/// what D19's equivalence law needs.
pub trait Clock {
    /// The current instant. Must be monotonic; a non-monotonic implementation is a
    /// port defect, and [`Instant::saturating_since`] keeps it from panicking a tick.
    fn now(&self) -> Instant;
}
