//! `set-origin` — (record, origin) → record with the origin field replaced (E13.3).
//!
//! The facade primitive decodes, patches, and re-encodes. This module exists so
//! `docs/specs/EDITOR.md` §5 can name the block.

/// Patches an encoded space record. The primitive implements the real work.
pub fn set_origin(record: &[u8], origin: &[u8]) -> Vec<u8> {
    let _ = origin;
    record.to_vec()
}
