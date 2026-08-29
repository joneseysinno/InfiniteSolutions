//! Mint a child address under a D45 parent (E13.4, O28 single-session answer).

use crate::editor::addresses;
use crate::facade::{significant_bits, Store};

/// The next free child key under `parent`, scanning the screen range.
pub fn next_child(store: &Store, parent: &[u8]) -> Option<Vec<u8>> {
    let parent_bits = significant_bits(parent);
    if parent_bits >= 28 {
        return None;
    }
    let child_bits = parent_bits + 4;
    let rows = store.records(addresses::SCREEN_ROOT_KEY, addresses::SCREEN_END_KEY);
    let mut max_idx = 0u8;
    for (key, _) in rows {
        if significant_bits(&key) != child_bits {
            continue;
        }
        if !shares_prefix(parent, parent_bits, &key) {
            continue;
        }
        max_idx = max_idx.max(child_nibble(parent_bits, &key));
    }
    let next = max_idx.checked_add(1)?;
    if next == 0 || next > 0x0F {
        return None;
    }
    Some(set_child_nibble(parent, parent_bits, next))
}

/// Parent to mint under: the hit if it hosts a space, else its containing parent.
pub fn placement_parent(store: &Store, hit: &[u8]) -> Vec<u8> {
    if hit.is_empty() {
        return addresses::CANVAS_KEY.to_vec();
    }
    if is_palette_item(hit) {
        return addresses::CANVAS_KEY.to_vec();
    }
    if let Some(payload) = store.stored_at(hit) {
        if let Some(space) = crate::facade::decode_space(&payload) {
            if space.hosts_space {
                return hit.to_vec();
            }
        }
    }
    parent_key(hit)
}

/// Whether a probe hit is a draggable palette template.
pub fn is_palette_item(key: &[u8]) -> bool {
    key.len() >= 2 && key[0] == addresses::PALETTE_KEY[0] && key[1] != 0
}

/// Drop position in the containing space's coordinates.
pub fn local_origin(store: &Store, parent: &[u8], x: f64, y: f64) -> Option<Vec<u8>> {
    let placement = store.place_now();
    let parent_rect = placement
        .placed
        .iter()
        .find(|p| p.at.as_bytes() == parent)
        .map(|p| p.rect)?;
    let w = (parent_rect.max.x - parent_rect.min.x).max(f64::MIN_POSITIVE);
    let h = (parent_rect.max.y - parent_rect.min.y).max(f64::MIN_POSITIVE);
    let local_x = ((x - parent_rect.min.x) / w).clamp(0.0, 1.0);
    let local_y = ((y - parent_rect.min.y) / h).clamp(0.0, 1.0);
    let mut out = Vec::with_capacity(16);
    out.extend_from_slice(&local_x.to_le_bytes());
    out.extend_from_slice(&local_y.to_le_bytes());
    Some(out)
}

/// Strip the last nibble — the containing space's key (D45).
pub fn parent_key(key: &[u8]) -> Vec<u8> {
    let bits = significant_bits(key);
    if bits <= significant_bits(addresses::CANVAS_KEY) {
        return addresses::CANVAS_KEY.to_vec();
    }
    let mut out = key.to_vec();
    let last = (bits / 4) as usize;
    write_nibble(&mut out, last, 0);
    for pos in (last + 1)..=8 {
        write_nibble(&mut out, pos, 0);
    }
    out
}

fn shares_prefix(parent: &[u8], parent_bits: u32, child: &[u8]) -> bool {
    let full_bytes = (parent_bits / 8) as usize;
    if parent[..full_bytes.min(parent.len())] != child[..full_bytes.min(child.len())] {
        return false;
    }
    let rem = parent_bits % 8;
    if rem == 0 {
        return true;
    }
    let mask = 0xFF_u8 << (8 - rem);
    (parent[full_bytes] & mask) == (child[full_bytes] & mask)
}

fn child_nibble(parent_bits: u32, key: &[u8]) -> u8 {
    read_nibble(key, (parent_bits / 4 + 1) as usize)
}

fn set_child_nibble(parent: &[u8], parent_bits: u32, index: u8) -> Vec<u8> {
    let mut out = parent.to_vec();
    let pos = (parent_bits / 4 + 1) as usize;
    write_nibble(&mut out, pos, index);
    for clear in (pos + 1)..=8 {
        write_nibble(&mut out, clear, 0);
    }
    out
}

fn read_nibble(key: &[u8], pos: usize) -> u8 {
    let byte_idx = (pos - 1) / 2;
    let byte = key.get(byte_idx).copied().unwrap_or(0);
    if pos % 2 == 1 {
        byte >> 4
    } else {
        byte & 0x0F
    }
}

fn write_nibble(key: &mut [u8], pos: usize, value: u8) {
    let byte_idx = (pos - 1) / 2;
    if byte_idx >= key.len() {
        return;
    }
    if pos % 2 == 1 {
        key[byte_idx] = (key[byte_idx] & 0x0F) | ((value & 0x0F) << 4);
    } else {
        key[byte_idx] = (key[byte_idx] & 0xF0) | (value & 0x0F);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_key_strips_the_last_nibble() {
        assert_eq!(
            parent_key(addresses::NODE_B_KEY),
            addresses::CANVAS_KEY.to_vec()
        );
        assert_eq!(
            parent_key(addresses::NODE_A1_KEY),
            addresses::NODE_A_KEY.to_vec()
        );
    }

    #[test]
    fn next_child_index_follows_siblings() {
        assert_eq!(set_child_nibble(addresses::CANVAS_KEY, 8, 4), {
            let mut k = addresses::CANVAS_KEY.to_vec();
            k[1] = 0x40;
            k
        });
    }
}
