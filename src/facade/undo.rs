//! Undo and redo (E12, D48).
//!
//! # The model, restated at the point it becomes code
//!
//! D48 settled the shape: undo never rewinds a revision, it writes the previous
//! value as a *new* commit. That has one consequence this file exists to satisfy —
//! if undo's own write entered the commit stream the way an ordinary edit does,
//! undoing twice would oscillate rather than converge. So the stream is not a
//! stack that undo pops and redo pushes back onto; it is one append-only log
//! (`Inner::commit_log`) with a cursor (`Inner::undo_cursor`) that undo moves back
//! and redo moves forward — and the write each performs is suppressed
//! (`SuppressUndo`) from ever extending the log itself. A fresh, ordinary commit
//! made while the cursor sits behind the end drops everything ahead of it
//! (E12.3) — see `super::ports::store_write`, the only place that ever pushes.
//!
//! # What is kept, and why it is a value, not a pointer
//!
//! The first draft of this file kept only `(address, revision)` pairs, meaning to
//! recover a value by asking `Inner::records_in_range` for "the revision before
//! this one" (R27 — no payload cache to keep in sync with the store's own
//! retained history). E12.0's own test caught why that does not work: the
//! revision `submit` can see at push time (`db.revision()`, and the `RevisionId`
//! `try_insert` returns) is a real wall-clock HLC stamp, while every `at` this
//! facade passes to a point-in-time read is the *dense* legacy sequence
//! `stable_revision()` returns — a different clock inside the same `RevisionId`
//! type (see `Inner::current_value`). Undo silently read the value it had just
//! written, because "revision minus one" on an HLC stamp is still, on the scale
//! `records_in_range` interprets it at, a revision far in the future. So each
//! entry instead carries the value it overwrote and the value it wrote, captured
//! synchronously at commit time, and reading history back out of `infinite-db` is
//! not needed at all.
//!
//! `revision` is still recorded, and still means the HLC-scale `db.revision()` —
//! [`Store::committed_since`] hands it out for ordering and watermark filtering
//! (E12.1's contract), and nothing here feeds it back into a point-in-time read.

use std::sync::{Arc, Mutex};

use infinite_runtime::binding::ArtifactRegistry;

use super::open::{Inner, Store};

/// The string key the undo stream registers under (D25, E12.5).
pub const UNDO_KEY: &str = "undo";

/// One commit this session made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitEntry {
    pub address: Vec<u8>,
    /// `db.revision()`'s HLC-scale stamp at commit time — an ordering and
    /// watermark-filter value only (E12.1). Never handed to a point-in-time read;
    /// see the module doc for why that would be a bug, not a shortcut.
    pub revision: u64,
    /// What `address` held immediately before this commit; `None` if this commit
    /// created it. Undo restores this, or tombstones the address if it is `None`.
    pub previous: Option<Vec<u8>>,
    /// What this commit wrote. Redo restores this.
    pub value: Vec<u8>,
}

/// RAII guard: while held, a write still marks `Inner::dirty` (staleness must see
/// every write, seed or not) but does not extend `Inner::commit_log` — see the
/// module doc and the field doc on `Inner::commit_log`.
pub(crate) struct SuppressUndo<'a> {
    flag: &'a Mutex<bool>,
}

impl<'a> SuppressUndo<'a> {
    pub(crate) fn engage(flag: &'a Mutex<bool>) -> Self {
        *flag.lock().expect("suppress_undo lock") = true;
        Self { flag }
    }
}

impl Drop for SuppressUndo<'_> {
    fn drop(&mut self) {
        *self.flag.lock().expect("suppress_undo lock") = false;
    }
}

impl Store {
    /// This session's commits since `watermark`, in commit order (E12.1).
    ///
    /// A genesis seed and undo/redo's own restoring write never appear here — see
    /// `Inner::commit_log`'s doc for why that is a second field rather than a
    /// filter over `dirty`. A pan never appears either, for the reason D48 gives
    /// directly: `pan_by`/`zoom_by` only ever `amend`, never `commit_at` (D5).
    pub fn committed_since(&self, watermark: u64) -> Vec<(Vec<u8>, u64)> {
        self.inner
            .commit_log
            .lock()
            .expect("commit_log lock")
            .iter()
            .filter(|e| e.revision > watermark)
            .map(|e| (e.address.clone(), e.revision))
            .collect()
    }

    /// Steps the undo cursor back one entry and commits the value read at the
    /// revision before that entry — as a **new** commit (D48 clause 1). Returns
    /// the address touched, or `None` if there is nothing left to undo.
    ///
    /// `stable_revision()` increases. Nothing is rewound.
    pub fn undo(&self) -> Option<Vec<u8>> {
        self.restore(true)
    }

    /// The mirror of [`Store::undo`]: re-commits the value the entry the cursor
    /// steps onto originally wrote. A fresh ordinary commit made in between drops
    /// everything redo could have reached (E12.3, `super::ports::store_write`).
    pub fn redo(&self) -> Option<Vec<u8>> {
        self.restore(false)
    }

    fn restore(&self, undoing: bool) -> Option<Vec<u8>> {
        let entry = {
            let log = self.inner.commit_log.lock().expect("commit_log lock");
            let mut cursor = self.inner.undo_cursor.lock().expect("undo_cursor lock");
            if undoing {
                if *cursor == 0 {
                    return None;
                }
                *cursor -= 1;
                log.get(*cursor).cloned()?
            } else {
                if *cursor >= log.len() {
                    return None;
                }
                let entry = log.get(*cursor).cloned()?;
                *cursor += 1;
                entry
            }
        };
        if undoing {
            // Undoing a create (`previous` is `None`) is committing a tombstone,
            // not a value — the general case, not a special one per verb (plan
            // §3.3). `delete_key`-style writes never enter `commit_log` in the
            // first place (`Inner::commit_pending` bypass, pre-existing), so this
            // branch cannot recurse into itself.
            match &entry.previous {
                Some(value) => self.put(&entry.address, value),
                None => {
                    let _suppress = SuppressUndo::engage(&self.inner.suppress_undo);
                    let _ = self
                        .inner
                        .db
                        .delete(self.inner.space, Inner::point_of(&entry.address));
                    let _ = self.sync();
                }
            }
        } else {
            self.put(&entry.address, &entry.value);
        }
        Some(entry.address)
    }
}

/// Registers the undo stream under [`UNDO_KEY`] (D25, E12.5).
///
/// Every other registered artifact's rebuild is a pure function of the *store*,
/// reached only through the `&dyn StoreRead` parameter — that is what the discard
/// harness (R12) is actually checking for them. This one cannot be: `commit_log`
/// is session state, not store-addressable (D8 has no room for it in Stored, and
/// R10 says a thing that only means something while running belongs to the
/// runtime — which is also O25's answer for why undo does not survive a
/// restart). So the rebuild closure captures `Inner` directly and ignores the
/// `store` argument entirely.
///
/// What R12 actually asks for — drop it, rebuild it, get identical bytes — still
/// holds: nothing except a new commit ever changes `commit_log`, and a rebuild
/// triggered by the harness is not a commit. If `commit_log` ever becomes
/// mutable concurrently with a rebuild the way store data legitimately is, this
/// registration stops being honest and should say so instead of quietly passing.
pub fn register_undo(registry: &mut ArtifactRegistry, inner: Arc<Inner>) {
    registry.register(UNDO_KEY, Vec::new(), move |_store| encode_commit_log(&inner));
}

fn encode_commit_log(inner: &Inner) -> Vec<u8> {
    let log = inner.commit_log.lock().expect("commit_log lock");
    let cursor = *inner.undo_cursor.lock().expect("undo_cursor lock");
    let mut out = Vec::new();
    out.extend_from_slice(&(cursor as u64).to_le_bytes());
    out.extend_from_slice(&(log.len() as u64).to_le_bytes());
    for entry in log.iter() {
        out.extend_from_slice(&(entry.address.len() as u32).to_le_bytes());
        out.extend_from_slice(&entry.address);
        out.extend_from_slice(&entry.revision.to_le_bytes());
        match &entry.previous {
            Some(p) => {
                out.push(1);
                out.extend_from_slice(&(p.len() as u32).to_le_bytes());
                out.extend_from_slice(p);
            }
            None => out.push(0),
        }
        out.extend_from_slice(&(entry.value.len() as u32).to_le_bytes());
        out.extend_from_slice(&entry.value);
    }
    out
}
