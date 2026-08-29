//! Property inspector — reads the selection through the facade's scene view (E13.2).
//! Inspector edits amend gesture addresses only; the behaviour composition writes
//! the selected space (E13.3).

use crate::editor::addresses;
use crate::facade::{encode_space, SpaceRecord, Store};

/// Rewrites the inspector's text fields from the current selection.
pub fn refresh(store: &Store) {
    let rows: [(&[u8], String); 6] = if let Some(view) = store.selection_view() {
        [
            (addresses::inspector_addr_key(), format!("addr {}", view.address)),
            (addresses::inspector_style_key(), format!("style {}", view.style)),
            (
                addresses::inspector_across_key(),
                format!(
                    "across {} {} {}",
                    view.across[0], view.across[1], view.across[2]
                ),
            ),
            (
                addresses::inspector_down_key(),
                format!("down {} {} {}", view.down[0], view.down[1], view.down[2]),
            ),
            (
                addresses::inspector_origin_key(),
                format!("origin {} {}", view.origin[0], view.origin[1]),
            ),
            (addresses::inspector_depth_key(), format!("depth {}", view.depth)),
        ]
    } else {
        [
            (addresses::inspector_addr_key(), "addr —".into()),
            (addresses::inspector_style_key(), "style —".into()),
            (addresses::inspector_across_key(), "across —".into()),
            (addresses::inspector_down_key(), "down —".into()),
            (addresses::inspector_origin_key(), "origin —".into()),
            (addresses::inspector_depth_key(), "depth —".into()),
        ]
    };
    for ((key, text), origin_y) in rows
        .iter()
        .zip([0.02, 0.08, 0.14, 0.20, 0.26, 0.32])
    {
        store.put(*key, &encode_space(&text_field(origin_y)));
        store.put_payload(key, text.as_bytes());
    }
}

/// Queues an origin edit. Only [`EDIT_ORIGIN_KEY`] and [`EDIT_COMMIT_KEY`] are
/// amended — never the selected node's record.
pub fn apply_origin(store: &Store, x: f64, y: f64) {
    let mut origin = Vec::with_capacity(16);
    origin.extend_from_slice(&x.to_le_bytes());
    origin.extend_from_slice(&y.to_le_bytes());
    store.amend(addresses::edit_origin_key(), &origin);
    store.amend(addresses::edit_commit_key(), &[1]);
}

fn text_field(origin_y: f64) -> SpaceRecord {
    SpaceRecord {
        across: [0.0, 0.0, 0.0],
        down: [0.0, 0.025, 0.0],
        style: "plain".into(),
        detail_override: None,
        hosts_space: false,
        accepts: false,
        origin: [0.02, origin_y],
        primitive: "text".into(),
    }
}
