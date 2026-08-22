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
