//! `gate` — pass a value through when a flag is set.

/// "On press, not on move."
pub fn gate(value: &[u8], flag: &[u8]) -> Option<Vec<u8>> {
    if flag.first().copied().unwrap_or(0) == 0 {
        return None;
    }
    Some(value.to_vec())
}
