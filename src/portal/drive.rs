//! The tick loop. Cadence in, work out (L1). Owns no thread pool.
//!
//! The event loop calls this; it never blocks, never sleeps, never spawns.

use crate::editor::{self, toolbar};
use crate::facade::Store;

/// Drives one tick from the event loop. Returns whether work remains.
pub fn drive(store: &Store) -> bool {
    editor::run(store);
    let work = if toolbar::graph_running(store) {
        store.tick().work_remains
    } else {
        false
    };
    editor::refresh_inspector(store);
    editor::refresh_toolbar(store);
    work
}
