//! [`Driver`] — cadence in, work out (L1, spec §7.1).

use super::ports::{Journal, JournalEntry, StaleFeed, StoreRead, StoreWrite, Submission};
use super::ArtifactRegistry;
use crate::core::{coalesce, Budget, Frontier, Instant, Outcome, PendingSet, Revision, Seq};

/// The driven runtime.
///
/// Holds the frontier, the pending set, the artifact registry, and the durable
/// watermark. Holds **no records** (R11), **no thread** (L1), and **no storage** (L2).
///
/// The ports are borrowed per tick rather than owned, so a `Driver` can be constructed
/// and inspected in a test with no store present at all — which is the point of D23.
#[derive(Debug)]
pub struct Driver {
    frontier: Frontier,
    pending: PendingSet,
    artifacts: ArtifactRegistry,
    watermark: Revision,
}

impl Driver {
    /// A driver with a pending set bounded at `pending_capacity` (R13).
    pub fn new(pending_capacity: usize) -> Self {
        Self {
            frontier: Frontier::new(),
            pending: PendingSet::new(pending_capacity),
            artifacts: ArtifactRegistry::new(),
            watermark: Revision::ZERO,
        }
    }

    /// The staleness frontier.
    pub fn frontier(&mut self) -> &mut Frontier {
        &mut self.frontier
    }

    /// The pending set. `list` on it is B4 — the "unsaved" indicator.
    pub fn pending(&mut self) -> &mut PendingSet {
        &mut self.pending
    }

    /// The artifact registry.
    pub fn artifacts(&mut self) -> &mut ArtifactRegistry {
        &mut self.artifacts
    }

    /// Restores pending state from the journal. Call once, before the first tick.
    ///
    /// A crash therefore loses at most the journal's unflushed tail (spec §7.3).
    /// Several amends to one origin collapse to the last: the pending set holds
    /// the gesture, not the keystroke history.
    pub fn replay(&mut self, journal: &dyn Journal) -> usize {
        let entries = journal.replay();
        let count = entries.len();
        let mut last = std::collections::BTreeMap::new();
        for entry in entries {
            last.insert(entry.origin.clone(), entry);
        }
        for entry in last.into_values() {
            if let Ok(seq) = self.pending.open(entry.origin, entry.payload) {
                if entry.committed {
                    self.pending.commit(seq);
                }
            }
        }
        count
    }

    /// Journals a pending entry. Every `amend` should be followed by one of these
    /// (D24.1) — appends are sequential and never touch the write queue.
    pub fn journal(&self, seq: Seq, journal: &mut dyn Journal) {
        if let Some(entry) = self.pending.list().find(|e| e.seq() == seq) {
            journal.append(&JournalEntry {
                seq,
                origin: entry.origin().clone(),
                payload: entry.payload().to_vec(),
                committed: entry.is_committed(),
            });
        }
    }

    /// One unit of cadence.
    ///
    /// **Never blocks, never sleeps, never spawns.** The caller decides whether to tick
    /// again, from [`Outcome::work_remains`].
    ///
    /// # Order of phases, and why
    ///
    /// 1. **Absorb staleness.** Before anything else, so nothing is rebuilt from a view
    ///    already known to be out of date.
    /// 2. **Submit commits.** Coalesced, and cheap because `submit` cannot block
    ///    (D24.4). Doing it before rebuilds means durability makes progress even in a
    ///    tick whose budget is entirely consumed by redraw work.
    /// 3. **Rebuild artifacts**, highest priority first, with whatever budget is left.
    ///    Last because it is the elastic phase — this is where a tight budget shows up
    ///    as fewer rebuilds rather than as a dropped keystroke.
    ///
    /// Note what is *not* here: nothing in any phase touches the input path. That is
    /// D24, and §7.4's saturation test is what proves it.
    pub fn tick(
        &mut self,
        now: Instant,
        mut budget: Budget,
        store_read: &dyn StoreRead,
        store_write: &mut dyn StoreWrite,
        stale: &dyn StaleFeed,
    ) -> Outcome {
        let mut outcome = Outcome::default();

        // 1 — absorb staleness.
        for (addr, at) in stale.stale_since(self.watermark) {
            for key in self.artifacts.invalidated_by(&addr) {
                self.artifacts.invalidate(&key);
            }
            self.frontier.mark(addr, at);
        }
        self.watermark = stale.watermark();

        // 2 — submit coalesced commits.
        let in_flight = store_write.in_flight();
        let decided = coalesce(self.pending.committed(), &in_flight);
        for seq in decided.superseded {
            self.pending.settle(seq);
        }
        for seq in decided.submit {
            if budget.is_exhausted(now) {
                outcome.budget_exhausted = true;
                break;
            }
            let Some(entry) = self.pending.list().find(|e| e.seq() == seq) else {
                continue;
            };
            let (origin, payload) = (entry.origin().clone(), entry.payload().to_vec());
            match store_write.submit(&origin, &payload) {
                Submission::Accepted => {
                    self.pending.settle(seq);
                    outcome.submitted += 1;
                }
                // Stays pending, retries next tick. The queue being full is not an
                // error and is never a wait.
                Submission::Full => outcome.refused += 1,
            }
            budget.spend();
        }

        // 3 — rebuild, highest priority first.
        while !budget.is_exhausted(now) {
            let Some((addr, _)) = self.frontier.take_next() else {
                break;
            };
            for key in self.artifacts.invalidated_by(&addr) {
                self.artifacts.rebuild(&key, store_read, self.watermark);
                outcome.rebuilt += 1;
            }
            budget.spend();
        }
        if budget.is_exhausted(now) && !self.frontier.is_empty() {
            outcome.budget_exhausted = true;
        }

        outcome.work_remains = !self.frontier.is_empty() || self.pending.committed().count() > 0;
        outcome
    }
}
