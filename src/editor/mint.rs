//! Mint a child address by pure derivation (E15 / O32).
//!
//! Significant length is carried with the key (`bits = 8 × byte length` under the
//! fixed-width slot encoding). Depth is never inferred by scanning for zero nibbles.

use crate::editor::addresses;
use crate::facade::Store;

/// Bits added per nesting level (one big-endian `u16` slot).
pub const SLOT_BITS: u32 = 16;

/// Slot `0` is reserved and never issued.
pub const SLOT_RESERVED: u32 = 0;

/// Maximum slot value (inclusive).
pub const SLOT_MAX: u32 = 0xFFFF;

/// Pure child derivation: append a 2-byte big-endian slot under `parent`.
///
/// `parent_bits` must equal `parent.len() * 8`. Slot `0` is refused.
pub fn child(parent: &[u8], parent_bits: u32, slot: u32) -> Option<(Vec<u8>, u32)> {
    if parent_bits != (parent.len() as u32).saturating_mul(8) {
        return None;
    }
    if slot == SLOT_RESERVED || slot > SLOT_MAX {
        return None;
    }
    if parent.len() + 2 > 15 {
        // Façade packs at most 15 key bytes (length trailer occupies byte 16).
        return None;
    }
    let mut out = parent.to_vec();
    out.push((slot >> 8) as u8);
    out.push((slot & 0xFF) as u8);
    let bits = parent_bits + SLOT_BITS;
    Some((out, bits))
}

/// Bits carried by a key under this encoding: eight per byte.
pub fn bits_of(key: &[u8]) -> u32 {
    (key.len() as u32).saturating_mul(8)
}

/// Stable slot from a local name (authored Spec / genesis). Never returns 0.
pub fn slot_for_name(name: &str) -> u32 {
    let mut h = 0x811c9dc5u32;
    for b in name.as_bytes() {
        h ^= u32::from(*b);
        h = h.wrapping_mul(0x0100_0193);
    }
    (h % SLOT_MAX) + 1
}

/// Session mint state — threads slots without reading the store (bion `IdSeed` shape).
#[derive(Debug, Clone, Copy)]
pub struct MintSeed {
    next: u32,
}

impl MintSeed {
    /// Starts issuing at slot 0x0100 — above typical authored Spec slots (1..N).
    pub fn new() -> Self {
        Self { next: 0x0100 }
    }

    /// Starts at an explicit next slot (tests / identical-machine seeds).
    pub fn at(next: u32) -> Self {
        Self {
            next: next.max(1).min(SLOT_MAX),
        }
    }

    /// Issues the next slot and advances. `None` when the space is full.
    pub fn next_slot(self) -> Option<(u32, Self)> {
        if self.next == 0 || self.next > SLOT_MAX {
            return None;
        }
        let slot = self.next;
        Some((slot, Self { next: slot + 1 }))
    }
}

impl Default for MintSeed {
    fn default() -> Self {
        Self::new()
    }
}

/// Mint the next child under `parent` from a seed. No store scan.
pub fn mint_child(parent: &[u8], seed: MintSeed) -> Option<(Vec<u8>, u32, MintSeed)> {
    let (slot, next) = seed.next_slot()?;
    let (bytes, bits) = child(parent, bits_of(parent), slot)?;
    Some((bytes, bits, next))
}

/// Parent to mint under: the hit if it hosts a space, else its containing parent.
pub fn placement_parent(store: &Store, hit: &[u8]) -> Vec<u8> {
    if hit.is_empty() {
        return addresses::canvas_key().to_vec();
    }
    if is_palette_item(hit) {
        return addresses::canvas_key().to_vec();
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
    key.len() > addresses::palette_key().len() && key.starts_with(addresses::palette_key())
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

/// Strip the last slot — the containing space's key.
pub fn parent_key(key: &[u8]) -> Vec<u8> {
    if key.len() <= addresses::canvas_key().len() {
        return addresses::canvas_key().to_vec();
    }
    if key.len() < 2 {
        return addresses::canvas_key().to_vec();
    }
    key[..key.len() - 2].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn child_appends_a_two_byte_slot() {
        let parent = &[0x10u8];
        let (bytes, bits) = child(parent, 8, 1).expect("slot 1");
        assert_eq!(bytes, vec![0x10, 0x00, 0x01]);
        assert_eq!(bits, 24);
        assert!(child(parent, 8, 0).is_none());
    }

    #[test]
    fn mint_seed_threads_without_io() {
        let mut seed = MintSeed::new();
        let mut seen = Vec::new();
        for _ in 0..200 {
            let (slot, next) = seed.next_slot().expect("slot");
            seen.push(slot);
            seed = next;
        }
        assert_eq!(seen.len(), 200);
        assert_eq!(seen[0], 0x0100);
        assert_eq!(seen[199], 0x0100 + 199);
        assert!(seen.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn two_seeds_do_not_share_slots_when_started_apart() {
        let a = MintSeed::at(0x0100);
        let b = MintSeed::at(10_000);
        let (sa, _) = a.next_slot().unwrap();
        let (sb, _) = b.next_slot().unwrap();
        assert_ne!(sa, sb);
    }

    #[test]
    fn parent_key_strips_the_last_slot() {
        let canvas = addresses::canvas_key();
        let (node, _) = child(canvas, bits_of(canvas), 1).unwrap();
        let (inner, _) = child(&node, bits_of(&node), 2).unwrap();
        assert_eq!(parent_key(&inner), node);
        assert_eq!(parent_key(&node), canvas.to_vec());
    }

    #[test]
    fn two_hundred_children_fit_under_one_parent() {
        let parent = addresses::canvas_key();
        let pb = bits_of(parent);
        let mut keys = Vec::new();
        for slot in 1..=200 {
            let (k, bits) = child(parent, pb, slot).expect("child");
            assert_eq!(bits, pb + SLOT_BITS);
            keys.push(k);
        }
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), 200);
    }
}
