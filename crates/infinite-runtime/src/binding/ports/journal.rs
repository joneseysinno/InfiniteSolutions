//! [`Journal`] — how pending state survives a crash without violating L2.

use crate::core::{Addr, Seq};

/// One journalled pending change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalEntry {
    /// The pending entry's sequence number.
    pub seq: Seq,
    /// The address the change is destined for.
    pub origin: Addr,
    /// The opaque payload (D13).
    pub payload: Vec<u8>,
    /// Whether the commit boundary had been crossed when this was written.
    pub committed: bool,
}

/// An append-only log of pending state.
///
/// # Why this is a port and not a file
///
/// L2 says the runtime owns no storage. Pending state is nonetheless the one
/// non-discardable thing it holds (D8), so it must survive a crash. The resolution is
/// that the runtime *calls* a journal and does not *implement* one: the facade points
/// this at the store's **session WAL**, a facility already built and, as of D8, still
/// unused.
///
/// Appends are sequential and never block the input path — that is the whole reason a
/// keystroke can be an `amend` rather than a write (D24.1).
pub trait Journal {
    /// Appends one entry.
    fn append(&mut self, entry: &JournalEntry);

    /// Everything the journal holds, oldest first.
    ///
    /// Called once before the first tick. Replay restores the pending set so that a
    /// crash loses at most the unflushed tail (spec §7.3).
    fn replay(&self) -> Vec<JournalEntry>;
}
