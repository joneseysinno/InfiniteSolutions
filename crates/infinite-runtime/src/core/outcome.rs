//! [`Outcome`] — what a tick did, and whether to tick again.

/// The report a `tick` returns (spec §7.1).
///
/// `work_remains` is the load-bearing field. Because the runtime owns no thread (L1),
/// the caller decides whether to tick again — so a tick that ran out of budget must be
/// able to say "not finished" without blocking, sleeping, or spawning.
///
/// This is also why O12 stays cheap. "May an iterative region yield between
/// iterations?" is, under this interface, already the default shape: a long
/// computation that honours the budget and reports `work_remains` needs no new
/// scheduler concept. The decision is still open; the interface does not foreclose it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    /// Artifacts rebuilt this tick.
    pub rebuilt: u32,
    /// Commits submitted to the store this tick.
    pub submitted: u32,
    /// Commits the store refused with `Full`. They stay pending and retry (D24.4).
    pub refused: u32,
    /// Whether the frontier or the pending set still has work.
    pub work_remains: bool,
    /// Whether the tick stopped because the budget ran out rather than because it
    /// finished. Sustained truth here is the signal that the cadence is too slow or
    /// the budget too small — and it is evidence for D19's compile decision, which is
    /// made on the runtime's evidence rather than the author's guess.
    pub budget_exhausted: bool,
}
