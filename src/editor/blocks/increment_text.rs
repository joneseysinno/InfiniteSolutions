//! `increment-text` — parse the text run as an integer, add one, write it back.

use crate::facade::{decode_space, encode_space};

/// Increments the numeric text payload of a space record.
pub fn increment_text(val: &[u8]) -> Vec<u8> {
    let Some(mut space) = decode_space(val) else {
        return val.to_vec();
    };
    let n: i64 = space.text.parse().unwrap_or(0);
    space.text = (n + 1).to_string();
    encode_space(&space)
}
