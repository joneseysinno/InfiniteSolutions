//! `encode-selection` — address bytes → an authored selection record.

/// Wraps a probed address as store bytes. Empty in, empty out.
pub fn encode_selection(hit: &[u8]) -> Vec<u8> {
    if hit.is_empty() {
        return Vec::new();
    }
    let mut out = b"SL1".to_vec();
    out.extend_from_slice(&(hit.len() as u16).to_le_bytes());
    out.extend_from_slice(hit);
    out
}
