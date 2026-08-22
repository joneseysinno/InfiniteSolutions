//! [`Journal`] — runtime. The session WAL, driven (D8).

use std::sync::Arc;

use infinite_db::infinitedb_core::address::DimensionVector;
use infinite_runtime::binding::ports::{Journal as Port, JournalEntry};
use infinite_runtime::core::{Addr, Seq};

use crate::facade::open::Inner;

/// The session-WAL-shaped journal.
pub struct Journal {
    pub(crate) inner: Arc<Inner>,
}

pub(crate) fn encode_entry(entry: &JournalEntry) -> Vec<u8> {
    let origin = entry.origin.as_bytes();
    let mut v = Vec::with_capacity(17 + origin.len() + entry.payload.len());
    v.extend_from_slice(&entry.seq.get().to_le_bytes());
    v.extend_from_slice(&(origin.len() as u32).to_le_bytes());
    v.extend_from_slice(origin);
    v.extend_from_slice(&(entry.payload.len() as u32).to_le_bytes());
    v.extend_from_slice(&entry.payload);
    v.push(u8::from(entry.committed));
    v
}

pub(crate) fn decode_entry(data: &[u8]) -> Option<JournalEntry> {
    if data.len() < 17 {
        return None;
    }
    let seq = u64::from_le_bytes(data[0..8].try_into().ok()?);
    let origin_len = u32::from_le_bytes(data[8..12].try_into().ok()?) as usize;
    let origin_at: usize = 12;
    let payload_len_at = origin_at.checked_add(origin_len)?;
    let payload_at = payload_len_at.checked_add(4)?;
    let origin = data.get(origin_at..payload_len_at)?.to_vec();
    let payload_len =
        u32::from_le_bytes(data.get(payload_len_at..payload_at)?.try_into().ok()?) as usize;
    let committed_at = payload_at.checked_add(payload_len)?;
    let payload = data.get(payload_at..committed_at)?.to_vec();
    let committed = *data.get(committed_at)? != 0;
    Some(JournalEntry {
        seq: Seq::new(seq),
        origin: Addr::new(origin),
        payload,
        committed,
    })
}

impl Port for Journal {
    fn append(&mut self, entry: &JournalEntry) {
        self.inner
            .journal
            .lock()
            .expect("journal lock")
            .push(entry.clone());
        let point = DimensionVector::new(vec![entry.seq.get() as u32]);
        let session = self.inner.journal_session();
        if let Err(e) = self.inner.db.insert_with_session(
            &session,
            self.inner.journal_space,
            point,
            encode_entry(entry),
        ) {
            panic!("journal append failed (not a silent drop): {e}");
        }
    }

    fn replay(&self) -> Vec<JournalEntry> {
        self.inner.journal.lock().expect("journal lock").clone()
    }
}
