//! Open the store.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::{Arc, Mutex};

use infinite_db::infinitedb_core::address::{DimensionVector, RevisionId, SpaceId};
use infinite_db::infinitedb_core::space::SpaceConfig;
use infinite_db::{EngineError, InfiniteDb, OpenOptions, WriteSession};
use infinite_runtime::binding::ports::JournalEntry;
use infinite_runtime::binding::Driver;
use infinite_presenter::core::{Camera, Placement, SurfaceRect};
use infinite_runtime::core::Addr;

use super::addr::runtime_addr;
use super::ports::{
    Backends, Blocks, Clock, Definitions, Glyphs, Journal, Provenance, Scene, StaleFeed, StoreRead,
    StoreWrite, Surface, Values,
};
use super::ports::decode_entry;

/// Shared handle onto an opened store. The thirteen ports borrow this.
pub struct Store {
    pub(crate) inner: Arc<Inner>,
}

pub(crate) struct Inner {
    pub db: InfiniteDb,
    pub space: SpaceId,
    pub journal_space: SpaceId,
    pub session: Mutex<Option<WriteSession>>,
    pub driver: Mutex<Driver>,
    pub in_flight: Mutex<BTreeSet<Addr>>,
    pub journal: Mutex<Vec<JournalEntry>>,
    pub surface: Mutex<SurfaceRect>,
    pub last_placement: Mutex<Option<Placement>>,
    pub findings: Mutex<Vec<infinite_compositor::core::Finding>>,
    pub slots: Mutex<BTreeMap<Addr, (String, Vec<u8>)>>,
    pub lineage: Mutex<BTreeMap<Addr, Vec<Addr>>>,
    pub dirty: Mutex<Vec<(Addr, u64)>>,
    pub plan: Mutex<Option<(Vec<u8>, Vec<u8>, Vec<u8>)>>,
    pub camera: Mutex<Option<Camera>>,
    pub graph_root: Mutex<Option<Vec<u8>>>,
    /// The authored style table's address range, handed over by the app (D44).
    pub style_range: Mutex<Option<(Vec<u8>, Vec<u8>)>>,
    /// The space whose fill is the background (E10.2). Authored, never a constant.
    pub background: Mutex<Option<Vec<u8>>>,
}

/// Opens the store at `dir`, registering the editor's 1-D space if needed.
pub fn open(dir: impl AsRef<Path>) -> Result<Store, EngineError> {
    open_with_options(dir, OpenOptions::default())
}

/// Opens with explicit options. The seam test uses a small write queue.
pub fn open_with_options(dir: impl AsRef<Path>, options: OpenOptions) -> Result<Store, EngineError> {
    let db = options.open(dir)?;
    let space = SpaceId(1);
    db.register_or_get_space(
        SpaceConfig::new(space, "editor", 1)
            .with_bits_per_dim(32)
            .with_shard_bits(0)
            .without_error_space(),
    )?;
    let journal_space = SpaceId(2);
    db.register_or_get_space(
        SpaceConfig::new(journal_space, "journal", 1)
            .with_bits_per_dim(32)
            .with_shard_bits(0)
            .without_error_space(),
    )?;
    let recovered = match db.query(journal_space, None) {
        Ok(rows) => rows
            .into_iter()
            .filter(|r| !r.tombstone)
            .filter_map(|r| decode_entry(&r.data))
            .collect(),
        Err(e) => return Err(e),
    };
    let store = Store {
        inner: Arc::new(Inner {
            db,
            space,
            journal_space,
            session: Mutex::new(None),
            driver: Mutex::new(Driver::new(1024)),
            in_flight: Mutex::new(BTreeSet::new()),
            journal: Mutex::new(recovered),
            surface: Mutex::new(SurfaceRect::new(
                infinite_presenter::core::Point::ORIGIN,
                infinite_presenter::core::Point::new(800.0, 600.0),
                1.0,
            )),
            last_placement: Mutex::new(None),
            findings: Mutex::new(Vec::new()),
            slots: Mutex::new(BTreeMap::new()),
            lineage: Mutex::new(BTreeMap::new()),
            dirty: Mutex::new(Vec::new()),
            plan: Mutex::new(None),
            camera: Mutex::new(None),
            graph_root: Mutex::new(None),
            style_range: Mutex::new(None),
            background: Mutex::new(None),
        }),
    };
    store.replay();
    Ok(store)
}

impl Store {
    /// Flushes the write queues so subsequent reads see committed values.
    pub fn sync(&self) -> Result<(), EngineError> {
        self.inner.db.sync()?;
        self.inner
            .in_flight
            .lock()
            .expect("in_flight lock")
            .clear();
        Ok(())
    }

    /// Stops shard I/O draining so [`StoreWrite::submit`] can return `Full` (D33).
    pub fn pause_write_drain(&self, pause: bool) {
        self.inner.db.pause_write_drain(pause);
    }

    /// Runtime range-read port.
    pub fn store_read(&self) -> StoreRead {
        StoreRead {
            inner: Arc::clone(&self.inner),
        }
    }

    /// Runtime submit port.
    pub fn store_write(&self) -> StoreWrite {
        StoreWrite {
            inner: Arc::clone(&self.inner),
        }
    }

    /// Runtime staleness port.
    pub fn stale_feed(&self) -> StaleFeed {
        StaleFeed {
            inner: Arc::clone(&self.inner),
        }
    }

    /// Runtime clock port.
    pub fn clock(&self) -> Clock {
        Clock::new()
    }

    /// Runtime journal port.
    pub fn journal(&self) -> Journal {
        Journal {
            inner: Arc::clone(&self.inner),
        }
    }

    /// Compositor definitions port (stored ∪ pending).
    pub fn definitions(&self) -> Definitions {
        Definitions {
            inner: Arc::clone(&self.inner),
        }
    }

    /// Compositor native-block registry.
    pub fn blocks(&self) -> Blocks {
        Blocks::new(Arc::clone(&self.inner))
    }

    /// Compositor values port.
    pub fn values(&self) -> Values {
        Values {
            inner: Arc::clone(&self.inner),
        }
    }

    /// Compositor provenance port.
    pub fn provenance(&self) -> Provenance {
        Provenance {
            inner: Arc::clone(&self.inner),
        }
    }

    /// Compositor backends port. Tier 0 after E9.
    pub fn backends(&self) -> Backends {
        Backends::new(Arc::clone(&self.inner))
    }

    /// Presenter scene port.
    pub fn scene(&self) -> Scene {
        Scene {
            inner: Arc::clone(&self.inner),
        }
    }

    /// Presenter surface. Geometry comes from the last resize (or the default).
    pub fn surface(&self) -> Surface {
        Surface::with_geometry(*self.inner.surface.lock().expect("surface lock"))
    }

    /// Presenter glyphs. Stub until a style names a font.
    pub fn glyphs(&self) -> Glyphs {
        Glyphs::new()
    }

    /// Whether a record exists at `key`.
    pub fn has(&self, key: &[u8]) -> bool {
        !self.records(key, &successor(key)).is_empty()
    }

    /// Writes one record and syncs.
    pub fn put(&self, key: &[u8], payload: &[u8]) {
        let mut write = self.store_write();
        use infinite_runtime::binding::ports::StoreWrite;
        let _ = write.submit(&runtime_addr(key), payload);
        drop(write);
        let _ = self.sync();
    }

    /// Tombstones one key and syncs.
    pub fn delete_key(&self, key: &[u8]) {
        let _ = self
            .inner
            .db
            .delete(self.inner.space, Inner::point_of(key));
        let _ = self.sync();
    }

    /// Tombstones every record in `[start, end)` and syncs.
    pub fn delete_range(&self, start: &[u8], end: &[u8]) {
        let keys: Vec<Vec<u8>> = self.records(start, end).into_iter().map(|(k, _)| k).collect();
        for key in keys {
            let _ = self
                .inner
                .db
                .delete(self.inner.space, Inner::point_of(&key));
        }
        let _ = self.sync();
    }

    /// Stored records in `[start, end)`, in address order.
    pub fn records(&self, start: &[u8], end: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
        let at = self.inner.db.stable_revision().legacy_sequence();
        match self.inner.records_in_range(start, end, at) {
            Ok(rows) => rows,
            Err(e) => panic!("store read failed (not an empty range): {e}"),
        }
    }

    /// Findings produced by the last place. An empty screen is one of them.
    pub fn last_findings(&self) -> Vec<infinite_compositor::core::Finding> {
        self.inner.findings.lock().expect("findings lock").clone()
    }

    /// R12 over a registered derived artifact, including the compiled form.
    pub fn artifact_passes_discard(&self, key: &str) -> bool {
        use infinite_runtime::binding::ports::StoreRead as StoreReadPort;
        let read = self.store_read();
        let at = StoreReadPort::head(&read);
        let mut driver = self.inner.driver.lock().expect("driver lock");
        driver.artifacts().passes_discard_test(key, &read, at)
    }
}

fn successor(key: &[u8]) -> Vec<u8> {
    let coord = Inner::coord(key).saturating_add(1);
    coord.to_be_bytes().to_vec()
}

impl Drop for Store {
    fn drop(&mut self) {
        self.inner.db.pause_write_drain(false);
    }
}

impl Inner {
    pub(crate) fn coord(bytes: &[u8]) -> u32 {
        if bytes.len() <= 4 {
            let mut buf = [0u8; 4];
            let n = bytes.len();
            if n > 0 {
                buf[4 - n..].copy_from_slice(&bytes[..n]);
            }
            return u32::from_be_bytes(buf);
        }
        fnv1a(bytes) | 0x8000_0000
    }

    pub(crate) fn bytes_of(coord: u32) -> Vec<u8> {
        coord.to_be_bytes().to_vec()
    }

    pub(crate) fn point_of(bytes: &[u8]) -> DimensionVector {
        DimensionVector::new(vec![Self::coord(bytes)])
    }

    pub(crate) fn records_in_range(
        &self,
        start: &[u8],
        end: &[u8],
        at: u64,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, EngineError> {
        let lo = Self::coord(start);
        let hi_excl = Self::coord(end);
        if hi_excl == 0 || lo >= hi_excl {
            return Ok(Vec::new());
        }
        let min = DimensionVector::new(vec![lo]);
        let max = DimensionVector::new(vec![hi_excl - 1]);
        let txn = self.db.read().as_of(RevisionId::legacy(at));
        match txn.query_bbox(self.space, min, max) {
            Ok(rows) => {
                let mut out: Vec<(Vec<u8>, Vec<u8>)> = rows
                    .into_iter()
                    .filter(|r| !r.tombstone)
                    .map(|r| {
                        let coord = r.address.point.coords.first().copied().unwrap_or(0);
                        (Self::bytes_of(coord), r.data)
                    })
                    .collect();
                out.sort_by(|a, b| a.0.cmp(&b.0));
                Ok(out)
            }
            Err(e) => Err(e),
        }
    }

    pub(crate) fn journal_session(&self) -> WriteSession {
        let mut slot = self.session.lock().expect("session lock");
        slot.get_or_insert_with(|| self.db.open_session()).clone()
    }

    pub(crate) fn overlay_pending(&self, start: &[u8], end: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
        let lo = runtime_addr(start);
        let hi = runtime_addr(end);
        self.driver
            .lock()
            .expect("driver lock")
            .pending()
            .list()
            .filter(|e| e.origin() >= &lo && e.origin() < &hi)
            .map(|e| (e.origin().as_bytes().to_vec(), e.payload().to_vec()))
            .collect()
    }

    pub(crate) fn amend_pending(inner: &Arc<Inner>, origin: &[u8], payload: &[u8]) {
        let origin = runtime_addr(origin);
        let mut driver = inner.driver.lock().expect("driver lock");
        let existing = driver
            .pending()
            .list()
            .find(|e| e.origin() == &origin && !e.is_committed())
            .map(|e| e.seq());
        let seq = if let Some(seq) = existing {
            if !driver.pending().amend(seq, payload.to_vec()) {
                return;
            }
            seq
        } else {
            match driver.pending().open(origin, payload.to_vec()) {
                Ok(seq) => seq,
                Err(_) => return,
            }
        };
        let mut journal = super::ports::Journal {
            inner: Arc::clone(inner),
        };
        driver.journal(seq, &mut journal);
    }

    pub(crate) fn commit_pending(inner: &Arc<Inner>, origin: &[u8]) -> bool {
        let origin = runtime_addr(origin);
        let mut driver = inner.driver.lock().expect("driver lock");
        let Some(seq) = driver
            .pending()
            .list()
            .find(|e| e.origin() == &origin && !e.is_committed())
            .map(|e| e.seq())
        else {
            return false;
        };
        let ok = driver.pending().commit(seq);
        if ok {
            let mut journal = super::ports::Journal {
                inner: Arc::clone(inner),
            };
            driver.journal(seq, &mut journal);
        }
        ok
    }
}

fn fnv1a(bytes: &[u8]) -> u32 {
    let mut h = 0x811c9dc5u32;
    for b in bytes {
        h ^= u32::from(*b);
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}
