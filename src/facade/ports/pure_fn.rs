//! Pure-function table (E18a). Variety is data; `map` / `fold` dispatch here.
//!
//! Each entry is a pure function of values — no store, no address, no ports.

use crate::editor::blocks::{
    displace as displace_fn, encode_selection as encode_selection_fn, increment_text,
    offset as offset_fn,
};
use crate::facade::{decode_space, encode_space};

/// Apply a registered pure-fn key to `val` and optional `aux`.
pub fn apply(key: &str, val: &[u8], aux: &[u8]) -> Vec<u8> {
    match key {
        "point-offset" => offset_fn(val, aux),
        "apply-delta" => apply_delta(val, aux),
        "set-origin" => set_origin(val, aux),
        "selection-bytes" => encode_selection_fn(val),
        "wire-space" => {
            if val.is_empty() || aux.is_empty() {
                Vec::new()
            } else {
                wire_space()
            }
        }
        "increment-text" => increment_text(val),
        "append-char" => append_char(val, aux),
        "add" => add_i64(val, aux),
        _ => val.to_vec(),
    }
}

/// Combine two values under a registered fold key (declared order: left then right).
pub fn fold_apply(key: &str, left: &[u8], right: &[u8]) -> Vec<u8> {
    apply(key, left, right)
}

fn apply_delta(record: &[u8], delta: &[u8]) -> Vec<u8> {
    let Some(mut space) = decode_space(record) else {
        return record.to_vec();
    };
    let next = displace_fn(
        &{
            let mut b = Vec::with_capacity(16);
            b.extend_from_slice(&space.origin[0].to_le_bytes());
            b.extend_from_slice(&space.origin[1].to_le_bytes());
            b
        },
        delta,
    );
    if next.len() >= 16 {
        space.origin[0] = f64::from_le_bytes(next[0..8].try_into().unwrap_or([0; 8]));
        space.origin[1] = f64::from_le_bytes(next[8..16].try_into().unwrap_or([0; 8]));
    }
    encode_space(&space)
}

fn set_origin(record: &[u8], origin: &[u8]) -> Vec<u8> {
    let Some(mut space) = decode_space(record) else {
        return record.to_vec();
    };
    if origin.len() >= 16 {
        space.origin[0] = f64::from_le_bytes(origin[0..8].try_into().unwrap_or([0; 8]));
        space.origin[1] = f64::from_le_bytes(origin[8..16].try_into().unwrap_or([0; 8]));
    }
    encode_space(&space)
}

fn wire_space() -> Vec<u8> {
    encode_space(&crate::facade::SpaceRecord {
        across: [0.012, 0.012, 0.0],
        down: [0.012, 0.012, 0.0],
        style: "wire".into(),
        detail_override: None,
        hosts_space: false,
        accepts: false,
        origin: [0.0, 0.0],
        primitive: "wire".into(),
    })
}

fn append_char(run: &[u8], key_ev: &[u8]) -> Vec<u8> {
    let mut s = String::from_utf8_lossy(run).into_owned();
    if let Some(ch) = key_to_char(key_ev) {
        if ch == '\u{8}' {
            s.pop();
        } else {
            s.push(ch);
        }
    }
    s.into_bytes()
}

fn add_i64(a: &[u8], b: &[u8]) -> Vec<u8> {
    let x: i64 = std::str::from_utf8(a).ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    let y: i64 = std::str::from_utf8(b).ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    (x + y).to_string().into_bytes()
}

/// Parse the portal key payload (`"{KeyCode:?}" ‖ pressed`).
pub fn key_to_char(payload: &[u8]) -> Option<char> {
    if payload.last().copied()? == 0 {
        return None;
    }
    let name = std::str::from_utf8(&payload[..payload.len() - 1]).ok()?;
    match name {
        "Backspace" => Some('\u{8}'),
        "Space" => Some(' '),
        "Digit0" | "Numpad0" => Some('0'),
        "Digit1" | "Numpad1" => Some('1'),
        "Digit2" | "Numpad2" => Some('2'),
        "Digit3" | "Numpad3" => Some('3'),
        "Digit4" | "Numpad4" => Some('4'),
        "Digit5" | "Numpad5" => Some('5'),
        "Digit6" | "Numpad6" => Some('6'),
        "Digit7" | "Numpad7" => Some('7'),
        "Digit8" | "Numpad8" => Some('8'),
        "Digit9" | "Numpad9" => Some('9'),
        s if s.starts_with("Key") && s.len() == 4 => s.chars().last().map(|c| c.to_ascii_lowercase()),
        _ => None,
    }
}
