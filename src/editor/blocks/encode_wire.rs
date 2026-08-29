//! `encode-wire` — (from, to) → encoded wire space record (E13.5).

/// Builds a wire record payload. The facade primitive performs the encoding.
pub fn encode_wire(from: &[u8], to: &[u8]) -> Vec<u8> {
    let _ = (from, to);
    Vec::new()
}
