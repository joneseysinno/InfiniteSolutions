//! [`PendingSet`] — the only non-discardable thing the runtime holds (D8, D24, R13).

use std::collections::BTreeMap;

use super::Addr;

/// A pending entry's identity. Monotonic, never reused (R17 at the value level).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Seq(u64);

impl Seq {
    /// Wraps a sequence number from a journal. A wrap, nothing else.
    pub const fn new(n: u64) -> Self {
        Self(n)
    }

    /// The raw sequence number, for journalling.
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// One uncommitted or committed-but-unsent change.
///
/// `payload` is **opaque** and its tag is the app's (D13). The runtime never parses,
/// validates, converts, compares or renders it — it stores it, moves it, and hands it
/// back.
#[derive(Clone, Debug)]
pub struct Pending {
    origin: Addr,
    payload: Box<[u8]>,
    seq: Seq,
    committed: bool,
}

impl Pending {
    /// The address this change is destined for.
    pub fn origin(&self) -> &Addr {
        &self.origin
    }

    /// The opaque payload.
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// This entry's sequence number.
    pub fn seq(&self) -> Seq {
        self.seq
    }

    /// Whether the commit boundary has been crossed.
    ///
    /// A `bool` rather than a two-variant enum, deliberately. D24 constructs the
    /// commit boundary as a single point — an entry is before it or after it, and
    /// there is no third position to add later. F-1 has occurred five times by
    /// admitting an enum whose set turned out to be open; this set is closed by
    /// construction, and saying so with a `bool` costs nothing and forecloses nothing.
    pub fn is_committed(&self) -> bool {
        self.committed
    }
}

/// What [`PendingSet::open`] reports when the set is full.
///
/// The policy is **commit the oldest, never drop** (spec §6). Dropping is not
/// available: the category is defined by not being discardable, so a bound that
/// discarded would be a bound on correctness rather than on memory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Overflow {
    /// The oldest entry, which the driver must commit and settle to make room.
    pub oldest: Seq,
}

/// Bounded, enumerable, non-discardable pending state.
///
/// Holds both halves of D24: uncommitted gestures (a half-typed value, a drag in
/// progress) and committed-but-unsent values awaiting an accepting write queue.
#[derive(Clone, Debug)]
pub struct PendingSet {
    entries: BTreeMap<Seq, Pending>,
    capacity: usize,
    next_seq: u64,
}

impl PendingSet {
    /// A set bounded at `capacity` entries.
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            capacity,
            next_seq: 0,
        }
    }

    /// Begins a gesture against `origin`.
    pub fn open(&mut self, origin: Addr, payload: impl Into<Box<[u8]>>) -> Result<Seq, Overflow> {
        if self.entries.len() >= self.capacity {
            let oldest = *self.entries.keys().next().expect("capacity > 0");
            return Err(Overflow { oldest });
        }
        let seq = Seq(self.next_seq);
        self.next_seq += 1;
        self.entries.insert(
            seq,
            Pending {
                origin,
                payload: payload.into(),
                seq,
                committed: false,
            },
        );
        Ok(seq)
    }

    /// Replaces the payload of an uncommitted entry. One keystroke, one `amend`.
    ///
    /// Returns `false` if the entry is gone or already past its commit boundary — an
    /// amendment to a committed value would be a write to something already destined
    /// for the store, which is a new gesture, not an edit to an old one.
    pub fn amend(&mut self, seq: Seq, payload: impl Into<Box<[u8]>>) -> bool {
        match self.entries.get_mut(&seq) {
            Some(entry) if !entry.committed => {
                entry.payload = payload.into();
                true
            }
            _ => false,
        }
    }

    /// Crosses the commit boundary. The entry stays pending until the store accepts it.
    pub fn commit(&mut self, seq: Seq) -> bool {
        match self.entries.get_mut(&seq) {
            Some(entry) if !entry.committed => {
                entry.committed = true;
                true
            }
            _ => false,
        }
    }

    /// Discards an uncommitted gesture. Committed entries cannot be abandoned.
    pub fn abandon(&mut self, seq: Seq) -> bool {
        match self.entries.get(&seq) {
            Some(entry) if !entry.committed => self.entries.remove(&seq).is_some(),
            _ => false,
        }
    }

    /// Removes an entry the store has accepted, or one superseded by a newer value for
    /// the same address (see [`coalesce`](super::coalesce)).
    pub fn settle(&mut self, seq: Seq) -> bool {
        self.entries.remove(&seq).is_some()
    }

    /// Everything pending, oldest first. This is B4 — an "unsaved" indicator is not
    /// implementable without it, and under D16's child constraint it is not optional.
    pub fn list(&self) -> impl Iterator<Item = &Pending> {
        self.entries.values()
    }

    /// Committed entries awaiting the store, oldest first. The driver's input to
    /// [`coalesce`](super::coalesce).
    pub fn committed(&self) -> impl Iterator<Item = &Pending> {
        self.entries.values().filter(|e| e.committed)
    }

    /// How many entries are pending.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether anything is pending.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The bound.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::{Addr, PendingSet};

    fn addr(b: u8) -> Addr {
        Addr::new(vec![b])
    }

    #[test]
    fn a_keystroke_amends_rather_than_accumulating() {
        let mut p = PendingSet::new(8);
        let seq = p.open(addr(1), b"1".to_vec()).unwrap();
        assert!(p.amend(seq, b"12".to_vec()));
        assert!(p.amend(seq, b"123".to_vec()));
        assert_eq!(p.len(), 1);
        assert_eq!(p.list().next().unwrap().payload(), b"123");
    }

    #[test]
    fn a_committed_entry_stays_pending_until_settled() {
        let mut p = PendingSet::new(8);
        let seq = p.open(addr(1), b"x".to_vec()).unwrap();
        assert!(p.commit(seq));
        assert_eq!(p.len(), 1, "committed-but-unsent is still pending (D24)");
        assert!(p.settle(seq));
        assert!(p.is_empty());
    }

    #[test]
    fn a_committed_entry_cannot_be_amended_or_abandoned() {
        let mut p = PendingSet::new(8);
        let seq = p.open(addr(1), b"x".to_vec()).unwrap();
        p.commit(seq);
        assert!(!p.amend(seq, b"y".to_vec()));
        assert!(!p.abandon(seq));
        assert_eq!(p.len(), 1);
    }

    #[test]
    fn overflow_names_the_oldest_and_drops_nothing() {
        let mut p = PendingSet::new(2);
        let first = p.open(addr(1), b"a".to_vec()).unwrap();
        p.open(addr(2), b"b".to_vec()).unwrap();
        let overflow = p.open(addr(3), b"c".to_vec()).unwrap_err();
        assert_eq!(overflow.oldest, first);
        assert_eq!(p.len(), 2, "nothing was dropped");
    }

    #[test]
    fn everything_pending_is_enumerable() {
        let mut p = PendingSet::new(8);
        p.open(addr(1), b"a".to_vec()).unwrap();
        let committed = p.open(addr(2), b"b".to_vec()).unwrap();
        p.commit(committed);
        assert_eq!(p.list().count(), 2);
        assert_eq!(p.committed().count(), 1);
    }
}
