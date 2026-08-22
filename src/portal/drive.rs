//! The tick loop. Cadence in, work out (L1). Owns no thread pool.
//!
//! The event loop calls this; it never blocks, never sleeps, never spawns.

use crate::editor;
use crate::facade::Store;

/// Drives one tick from the event loop. Returns whether work remains.
pub fn drive(store: &Store) -> bool {
    editor::run(store);
    store.tick().work_remains
}
