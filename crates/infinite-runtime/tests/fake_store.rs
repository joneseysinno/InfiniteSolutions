//! The only store this layer ever names (D23).
//!
//! Not a mock of `infinite-db` — a store with the one property the real one has that
//! matters here: a write queue that can be **filled on demand**. A real store's queue
//! cannot be saturated to order, which is why §7.4's test could not exist without this
//! file. That is the concrete payoff of the runtime depending on no layer.
//!
//! `FakeStore` is a cheap handle over shared state, so a test can hold one as
//! `&dyn StoreRead` and another as `&mut dyn StoreWrite` at the same time.

#![cfg(feature = "binding")]
#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use infinite_runtime::binding::ports::{
    Clock, Journal, JournalEntry, Records, StaleFeed, StoreRead, StoreWrite, Submission,
};
use infinite_runtime::core::{Addr, Instant, Revision};

#[derive(Default)]
struct Inner {
    records: BTreeMap<Addr, Vec<u8>>,
    stale: Vec<(Addr, Revision)>,
    head: Revision,
    queue: Vec<(Addr, Vec<u8>)>,
    queue_capacity: usize,
    reads: usize,
}

/// A store whose write queue has a settable capacity.
#[derive(Clone, Default)]
pub struct FakeStore(Rc<RefCell<Inner>>);

impl FakeStore {
    /// A store with a write queue of `queue_capacity` entries.
    pub fn new(queue_capacity: usize) -> Self {
        Self(Rc::new(RefCell::new(Inner {
            queue_capacity,
            ..Default::default()
        })))
    }

    /// A second handle onto the same state.
    pub fn handle(&self) -> Self {
        Self(Rc::clone(&self.0))
    }

    /// Seeds a record without going through the queue.
    pub fn seed(&self, addr: Addr, payload: Vec<u8>) {
        self.0.borrow_mut().records.insert(addr, payload);
    }

    /// Announces that `addr` went stale at the next revision.
    pub fn go_stale(&self, addr: Addr) {
        let mut inner = self.0.borrow_mut();
        inner.head = Revision::new(inner.head.get() + 1);
        let head = inner.head;
        inner.stale.push((addr, head));
    }

    /// Applies everything queued. Stands in for the store's writer making progress.
    pub fn drain(&self) {
        let mut inner = self.0.borrow_mut();
        for (addr, payload) in std::mem::take(&mut inner.queue) {
            inner.records.insert(addr, payload);
        }
    }

    /// Fills the write queue to capacity, so every `submit` returns `Full`.
    pub fn saturate(&self) {
        let mut inner = self.0.borrow_mut();
        let filler = Addr::new(vec![0xFF, 0xFF]);
        while inner.queue.len() < inner.queue_capacity {
            inner.queue.push((filler.clone(), Vec::new()));
        }
    }

    /// Whether the queue is full — the condition §7.4 holds true for 10 seconds.
    pub fn is_saturated(&self) -> bool {
        let inner = self.0.borrow();
        inner.queue.len() >= inner.queue_capacity
    }

    /// A committed record's current value.
    pub fn get(&self, addr: &Addr) -> Option<Vec<u8>> {
        self.0.borrow().records.get(addr).cloned()
    }

    /// How many range reads have been served.
    pub fn reads(&self) -> usize {
        self.0.borrow().reads
    }
}

impl StoreRead for FakeStore {
    fn range(&self, start: &Addr, end: &Addr, _at: Revision) -> Records {
        let mut inner = self.0.borrow_mut();
        inner.reads += 1;
        inner
            .records
            .range(start.clone()..end.clone())
            .map(|(a, p)| (a.clone(), p.clone()))
            .collect()
    }

    fn head(&self) -> Revision {
        self.0.borrow().head
    }
}

impl StoreWrite for FakeStore {
    fn submit(&mut self, origin: &Addr, payload: &[u8]) -> Submission {
        // The whole point: returns, never waits (D24.4).
        let mut inner = self.0.borrow_mut();
        if inner.queue.len() >= inner.queue_capacity {
            return Submission::Full;
        }
        inner.queue.push((origin.clone(), payload.to_vec()));
        Submission::Accepted
    }

    fn in_flight(&self) -> BTreeSet<Addr> {
        self.0
            .borrow()
            .queue
            .iter()
            .map(|(a, _)| a.clone())
            .collect()
    }
}

impl StaleFeed for FakeStore {
    fn stale_since(&self, watermark: Revision) -> Vec<(Addr, Revision)> {
        self.0
            .borrow()
            .stale
            .iter()
            .filter(|(_, at)| *at > watermark)
            .cloned()
            .collect()
    }

    fn watermark(&self) -> Revision {
        self.0.borrow().head
    }
}

/// A clock that only moves when a test moves it.
#[derive(Default)]
pub struct FakeClock(std::cell::Cell<u64>);

impl FakeClock {
    /// Advances by `nanos`.
    pub fn advance(&self, nanos: u64) {
        self.0.set(self.0.get() + nanos);
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Instant {
        Instant::from_nanos(self.0.get())
    }
}

/// An in-memory journal.
#[derive(Default)]
pub struct FakeJournal {
    entries: Vec<JournalEntry>,
    flushed: usize,
}

impl FakeJournal {
    /// Marks everything appended so far as durable.
    pub fn flush(&mut self) {
        self.flushed = self.entries.len();
    }

    /// Entries appended but not yet durable — what a crash would lose (§7.3).
    pub fn unflushed(&self) -> usize {
        self.entries.len() - self.flushed
    }
}

impl Journal for FakeJournal {
    fn append(&mut self, entry: &JournalEntry) {
        self.entries.push(entry.clone());
    }

    fn replay(&self) -> Vec<JournalEntry> {
        self.entries[..self.flushed].to_vec()
    }
}
