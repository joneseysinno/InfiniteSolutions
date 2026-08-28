//! [`StoreWrite`] — runtime. Submit; returns `Accepted` or `Full`, never blocks (D24, D33).

use std::sync::Arc;

use infinite_db::EngineError;
use infinite_runtime::binding::ports::{StoreWrite as Port, Submission};
use infinite_runtime::core::Addr;

use crate::facade::open::Inner;

/// Non-blocking submit over the real store.
pub struct StoreWrite {
    pub(crate) inner: Arc<Inner>,
}

impl Port for StoreWrite {
    fn submit(&mut self, origin: &Addr, payload: &[u8]) -> Submission {
        let point = Inner::point_of(origin.as_bytes());
        // Captured *before* the write, and only when it will actually be used
        // (E12): a genesis seed or undo/redo's own restoring write is suppressed
        // from the undo stream, so there is no reason to pay for the read.
        let suppressed = *self.inner.suppress_undo.lock().expect("suppress_undo lock");
        let previous = if suppressed {
            None
        } else {
            Some(self.inner.current_value(origin.as_bytes()))
        };
        match self
            .inner
            .db
            .try_insert(self.inner.space, point, payload.to_vec())
        {
            Ok(_) => {
                self.inner
                    .in_flight
                    .lock()
                    .expect("in_flight lock")
                    .insert(origin.clone());
                let rev = self.inner.db.revision().legacy_sequence();
                self.inner
                    .dirty
                    .lock()
                    .expect("dirty lock")
                    .push((origin.clone(), rev));
                // E12: extend the undo stream, unless this write is suppressed. A
                // commit made with the cursor sitting behind the end drops the
                // redo tail ahead of it (E12.3) before appending — the one place
                // the tail is ever dropped.
                if let Some(previous) = previous {
                    let mut log = self.inner.commit_log.lock().expect("commit_log lock");
                    let mut cursor = self.inner.undo_cursor.lock().expect("undo_cursor lock");
                    log.truncate(*cursor);
                    log.push(crate::facade::undo::CommitEntry {
                        address: origin.as_bytes().to_vec(),
                        revision: rev,
                        previous,
                        value: payload.to_vec(),
                    });
                    *cursor = log.len();
                }
                Submission::Accepted
            }
            Err(EngineError::QueueFull) => Submission::Full,
            Err(e) => panic!("store write failed: {e}"),
        }
    }

    fn in_flight(&self) -> std::collections::BTreeSet<Addr> {
        self.inner
            .in_flight
            .lock()
            .expect("in_flight lock")
            .clone()
    }
}
