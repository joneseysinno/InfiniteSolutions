//! [`StoreRead`] — runtime. Range reads at a revision.

use std::sync::Arc;

use infinite_runtime::binding::ports::{Records, StoreRead as Port};
use infinite_runtime::core::{Addr, Revision};

use crate::facade::addr::runtime_addr;
use crate::facade::open::Inner;

/// Range reads over the real store.
pub struct StoreRead {
    pub(crate) inner: Arc<Inner>,
}

impl Port for StoreRead {
    fn range(&self, start: &Addr, end: &Addr, at: Revision) -> Records {
        match self
            .inner
            .records_in_range(start.as_bytes(), end.as_bytes(), at.get())
        {
            Ok(rows) => rows
                .into_iter()
                .map(|(bytes, payload)| (runtime_addr(&bytes), payload))
                .collect(),
            Err(e) => panic!("store read failed (not an empty range): {e}"),
        }
    }

    fn head(&self) -> Revision {
        crate::facade::addr::runtime_revision(self.inner.db.stable_revision().legacy_sequence())
    }
}
