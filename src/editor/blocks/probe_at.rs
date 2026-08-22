//! `probe-at` — a point → an address.

/// Decodes a point payload. The facade Primitive is what hits the placement
/// (only the presenter can). This file names no layer crate.
pub fn probe_at(point: &[u8]) -> Option<[u8; 16]> {
    point.get(..16)?.try_into().ok()
}
