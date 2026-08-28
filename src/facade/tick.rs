//! Cadence in, work out. The facade's drive surface, so the portal never names a layer.

use infinite_db::EngineError;
use infinite_runtime::binding::ports::Clock as ClockPort;
use infinite_runtime::core::{Budget, Instant};

use super::addr::runtime_addr;
use super::open::Store;
use super::ports::{Clock, Journal, StaleFeed, StoreRead, StoreWrite};

/// What one tick did. A facade type, so `src/portal/` does not name the runtime.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TickReport {
    /// Commits the store accepted this tick.
    pub submitted: u32,
    /// Commits the store refused with `Full`.
    pub refused: u32,
    /// Whether the frontier or the pending set still has work.
    pub work_remains: bool,
    /// Whether the tick stopped because the budget ran out.
    pub budget_exhausted: bool,
}

impl Store {
    /// Amends a pending entry at `origin`. Never a store write (D24).
    ///
    /// Returns whether the keystroke landed in the pending set.
    pub fn amend(&self, origin: &[u8], payload: &[u8]) -> bool {
        let origin = runtime_addr(origin);
        let mut driver = self.inner.driver.lock().expect("driver lock");
        let existing = driver
            .pending()
            .list()
            .find(|e| e.origin() == &origin && !e.is_committed())
            .map(|e| e.seq());
        let seq = if let Some(seq) = existing {
            if !driver.pending().amend(seq, payload.to_vec()) {
                return false;
            }
            seq
        } else {
            match driver.pending().open(origin, payload.to_vec()) {
                Ok(seq) => seq,
                Err(overflow) => {
                    driver.pending().commit(overflow.oldest);
                    return false;
                }
            }
        };
        let mut journal = Journal {
            inner: std::sync::Arc::clone(&self.inner),
        };
        driver.journal(seq, &mut journal);
        true
    }

    /// Crosses the commit boundary for the pending entry at `origin`.
    ///
    /// The interpreted composition's native `commit` block reaches the same
    /// action through `Inner::commit_pending` (`facade/ports/blocks.rs`) — this is
    /// the app-driven entry to it, not a second implementation of it (R27/F-3).
    pub fn commit_at(&self, origin: &[u8]) -> bool {
        super::open::Inner::commit_pending(&self.inner, origin)
    }

    /// Discards the pending gesture at `origin`, if any and if it has not yet
    /// crossed the commit boundary. The other verb (D48 clause 2, E12.4): only
    /// `super::ports::store_write`'s `submit` ever extends `Inner::commit_log`, and
    /// `abandon` never reaches it, so a discard can never add an undo entry.
    pub fn discard_at(&self, origin: &[u8]) -> bool {
        let origin = runtime_addr(origin);
        let mut driver = self.inner.driver.lock().expect("driver lock");
        let Some(seq) = driver
            .pending()
            .list()
            .find(|e| e.origin() == &origin && !e.is_committed())
            .map(|e| e.seq())
        else {
            return false;
        };
        driver.pending().abandon(seq)
    }

    /// One unit of cadence. Reads the clock; never blocks, never sleeps, never spawns.
    pub fn tick(&self) -> TickReport {
        let now = Clock::new().now();
        self.tick_at(now.as_nanos(), 8)
    }

    /// One unit of cadence at an explicit instant, for tests (D19 reproducibility).
    pub fn tick_at(&self, now_nanos: u64, budget_units: u32) -> TickReport {
        let mut driver = self.inner.driver.lock().expect("driver lock");
        let read = StoreRead {
            inner: std::sync::Arc::clone(&self.inner),
        };
        let mut write = StoreWrite {
            inner: std::sync::Arc::clone(&self.inner),
        };
        let stale = StaleFeed {
            inner: std::sync::Arc::clone(&self.inner),
        };
        let outcome = driver.tick(
            Instant::from_nanos(now_nanos),
            Budget::units(budget_units),
            &read,
            &mut write,
            &stale,
        );
        TickReport {
            submitted: outcome.submitted,
            refused: outcome.refused,
            work_remains: outcome.work_remains,
            budget_exhausted: outcome.budget_exhausted,
        }
    }

    /// Restores the pending set from the journal. Called from [`super::open`].
    pub fn replay(&self) -> usize {
        let mut driver = self.inner.driver.lock().expect("driver lock");
        let journal = Journal {
            inner: std::sync::Arc::clone(&self.inner),
        };
        driver.replay(&journal)
    }

    /// How many pending entries are held. B4 — the "unsaved" indicator.
    pub fn pending_len(&self) -> usize {
        self.inner
            .driver
            .lock()
            .expect("driver lock")
            .pending()
            .len()
    }

    /// The newest payload pending at `origin`, if any.
    pub fn pending_at(&self, origin: &[u8]) -> Option<Vec<u8>> {
        let origin = runtime_addr(origin);
        let mut driver = self.inner.driver.lock().expect("driver lock");
        let found = driver
            .pending()
            .list()
            .filter(|e| e.origin() == &origin)
            .last()
            .map(|e| e.payload().to_vec());
        found
    }

    /// How many committed-but-unsent entries remain.
    pub fn committed_len(&self) -> usize {
        self.inner
            .driver
            .lock()
            .expect("driver lock")
            .pending()
            .committed()
            .count()
    }

    /// The stored payload at `origin` after a drain, for the saturation test.
    pub fn stored_at(&self, origin: &[u8]) -> Option<Vec<u8>> {
        let end = {
            let mut c = crate::facade::open::Inner::coord(origin);
            c = c.saturating_add(1);
            crate::facade::open::Inner::bytes_of(c)
        };
        let at = self.inner.db.stable_revision().legacy_sequence();
        match self.inner.records_in_range(origin, &end, at) {
            Ok(mut rows) => rows.pop().map(|(_, payload)| payload),
            Err(e) => panic!("store read failed (not a missing value): {e}"),
        }
    }

    /// Fsyncs the session WAL so a crash loses at most the unflushed tail (D8).
    pub fn flush_journal(&self) -> Result<(), EngineError> {
        let session = {
            let slot = self.inner.session.lock().expect("session lock");
            match slot.as_ref() {
                Some(session) => session.clone(),
                None => return Ok(()),
            }
        };
        let durable = self.inner.db.sync_session_wal(&session)?;
        if session.has_pending_intent() {
            self.inner
                .db
                .commit_session_intent(&session, &durable)?;
        }
        Ok(())
    }
}
