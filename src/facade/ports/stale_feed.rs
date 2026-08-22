//! [`StaleFeed`] — runtime. Staleness closure → "these went stale at rev N".

use std::sync::Arc;

use infinite_runtime::binding::ports::StaleFeed as Port;
use infinite_runtime::core::{Addr, Revision};

use crate::facade::addr::runtime_revision;
use crate::facade::open::Inner;

/// Staleness over the real store. Downstream addresses come from the provenance
/// the compositor recorded (D11, D38).
pub struct StaleFeed {
    pub(crate) inner: Arc<Inner>,
}

impl Port for StaleFeed {
    fn stale_since(&self, watermark: Revision) -> Vec<(Addr, Revision)> {
        let dirty = self.inner.dirty.lock().expect("dirty lock");
        dirty
            .iter()
            .filter(|(_, at)| *at > watermark.get())
            .map(|(addr, at)| (addr.clone(), runtime_revision(*at)))
            .collect()
    }

    fn watermark(&self) -> Revision {
        runtime_revision(self.inner.db.revision().legacy_sequence())
    }
}
