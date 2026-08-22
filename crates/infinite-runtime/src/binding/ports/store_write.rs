//! [`StoreWrite`] — where O3 is closed (D24).

use crate::core::Addr;

/// What the store said about a submitted commit.
///
/// A two-state result, and a plain enum rather than a registry because this set is
/// closed by D24's construction: `submit` either takes the commit or it does not.
/// R16 forbids a closed enum *wherever the set is open*; this one is not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Submission {
    /// The store took it. The entry may be settled.
    Accepted,
    /// The write queue is full. The entry stays pending and retries on a later tick.
    /// **This is a return value, not a wait.**
    Full,
}

/// Submits commits to the store.
///
/// # The one thing this port must never do
///
/// **`submit` must not block.** `infinite-db`'s write queue blocks when full; that
/// blocking stops at this boundary and is reported as [`Submission::Full`] instead.
///
/// If it blocked, a full queue would stall the tick, the tick would stall the cadence,
/// and the cadence would stall typing — which is exactly O3, and exactly what R14
/// forbids. Because it returns instead, backpressure degrades **durability latency**
/// (how soon a committed value is safe) rather than **input latency**. That is the
/// correct thing to degrade, and choosing which one to trade is the whole substance of
/// D24.
///
/// An implementation that waits is a defect in the facade, not in the runtime — but
/// the runtime is the thing that breaks, so §7.4's saturation test lives here.
pub trait StoreWrite {
    /// Offers one commit. Returns immediately, always.
    fn submit(&mut self, origin: &Addr, payload: &[u8]) -> Submission;

    /// Addresses with a commit currently in flight.
    ///
    /// Feeds [`coalesce`](crate::core::coalesce), which holds at most one commit per
    /// address (D24.3).
    fn in_flight(&self) -> std::collections::BTreeSet<Addr>;
}
