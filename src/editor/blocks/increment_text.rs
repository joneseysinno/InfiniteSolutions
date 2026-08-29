//! `increment-text` — parse a text payload as an integer, add one, write it back.

/// Increments a UTF-8 numeric payload (E17: not a `SpaceRecord` field).
pub fn increment_text(val: &[u8]) -> Vec<u8> {
    let s = std::str::from_utf8(val).unwrap_or("0");
    let n: i64 = s.trim().parse().unwrap_or(0);
    (n + 1).to_string().into_bytes()
}
