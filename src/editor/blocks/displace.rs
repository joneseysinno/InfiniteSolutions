//! `displace` — (origin, delta) → origin.

/// Applies a point-delta to an authored origin. Drag requires this: `offset`
/// produces the delta, and without a block that applies it `amend` would have to
/// interpret a point as a record patch — two jobs in one block.
pub fn displace(origin: &[u8], delta: &[u8]) -> Vec<u8> {
    let Some(a) = point(origin) else {
        return origin.to_vec();
    };
    let Some(d) = point(delta) else {
        return origin.to_vec();
    };
    let mut out = Vec::with_capacity(16);
    out.extend_from_slice(&(a[0] + d[0]).to_le_bytes());
    out.extend_from_slice(&(a[1] + d[1]).to_le_bytes());
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
