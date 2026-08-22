//! `offset` — (point, point) → point.

/// A drag is a delta. Every number is inside a block (L3).
pub fn offset(from: &[u8], to: &[u8]) -> Vec<u8> {
    let Some(a) = point(from) else {
        return Vec::new();
    };
    let Some(b) = point(to) else {
        return Vec::new();
    };
    let mut out = Vec::with_capacity(16);
    out.extend_from_slice(&(b[0] - a[0]).to_le_bytes());
    out.extend_from_slice(&(b[1] - a[1]).to_le_bytes());
    out
}

fn point(bytes: &[u8]) -> Option<[f64; 2]> {
    if bytes.len() < 16 {
        return None;
    }
    Some([
        f64::from_le_bytes(bytes[0..8].try_into().ok()?),
        f64::from_le_bytes(bytes[8..16].try_into().ok()?),
    ])
}
