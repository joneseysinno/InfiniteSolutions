//! Block palette — appearance in genesis; drop gesture wired in [`crate::editor::run`].

use crate::editor::addresses;
use crate::editor::mint;
use crate::facade::Store;

/// Whether `key` is under the palette region.
pub fn is_palette_item(key: &[u8]) -> bool {
    mint::is_palette_item(key)
}

/// Palette template under a surface point. Uses placement geometry because
/// [`Store::probe_at`] drills into the canvas and misses palette siblings (D45).
pub fn hit_at(store: &Store, x: f64, y: f64) -> Option<Vec<u8>> {
    let placement = store.place_now();
    placement
        .placed
        .iter()
        .filter(|p| {
            is_palette_item(p.at.as_bytes())
                && p.accepts
                && x >= p.rect.min.x
                && x <= p.rect.max.x
                && y >= p.rect.min.y
                && y <= p.rect.max.y
        })
        .last()
        .map(|p| p.at.as_bytes().to_vec())
}

/// Resolves the parent space and mints a child address at a surface point.
pub fn prepare_drop(store: &Store, surface_x: f64, surface_y: f64, hit: &[u8]) -> bool {
    let parent = mint::placement_parent(store, hit);
    let Some(addr) = mint::next_child(store, &parent) else {
        return false;
    };
    let Some(origin) = mint::local_origin(store, &parent, surface_x, surface_y) else {
        return false;
    };
    store.amend(addresses::PLACE_ADDR_KEY, &addr);
    store.amend(addresses::PLACE_ORIGIN_KEY, &origin);
    store.amend(addresses::PLACE_COMMIT_KEY, &[1]);
    true
}
