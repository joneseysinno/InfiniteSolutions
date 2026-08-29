//! Toolbar — three affordances that survive §4's test (E13.6).
//!
//! Undo/redo, zoom readout, and run/pause. Appearance is authored spaces; clicks
//! call store verbs directly, like the portal's key bindings (E12.6).

use crate::editor::addresses;
use crate::facade::{encode_space, SpaceRecord, Store};

/// Whether `key` is under the toolbar region.
pub fn is_toolbar_item(key: &[u8]) -> bool {
    key.len() >= 2 && key[0] == addresses::TOOLBAR_KEY[0] && key[1] != 0
}

/// Toolbar hit at a surface point. Uses placement geometry like the palette (D45).
pub fn hit_at(store: &Store, x: f64, y: f64) -> Option<Vec<u8>> {
    let placement = store.place_now();
    placement
        .placed
        .iter()
        .filter(|p| {
            is_toolbar_item(p.at.as_bytes())
                && p.accepts
                && x >= p.rect.min.x
                && x <= p.rect.max.x
                && y >= p.rect.min.y
                && y <= p.rect.max.y
        })
        .last()
        .map(|p| p.at.as_bytes().to_vec())
}

/// Session run flag. Defaults to running when unset.
pub fn graph_running(store: &Store) -> bool {
    store
        .pending_at(addresses::RUN_KEY)
        .or_else(|| store.stored_at(addresses::RUN_KEY))
        .map(|b| b.first().copied().unwrap_or(1) != 0)
        .unwrap_or(true)
}

/// Handles a toolbar click at a surface point on pointer release.
pub fn activate(store: &Store, x: f64, y: f64) {
    let Some(hit) = hit_at(store, x, y) else {
        return;
    };
    if hit == addresses::TOOLBAR_HISTORY_KEY {
        let placement = store.place_now();
        let item = placement
            .placed
            .iter()
            .find(|p| p.at.as_bytes() == addresses::TOOLBAR_HISTORY_KEY)
            .expect("history affordance is placed");
        let mid_x = (item.rect.min.x + item.rect.max.x) * 0.5;
        if x < mid_x {
            let _ = store.undo();
        } else {
            let _ = store.redo();
        }
    } else if hit == addresses::TOOLBAR_RUN_KEY {
        let running = graph_running(store);
        store.amend(addresses::RUN_KEY, &[u8::from(!running)]);
        refresh(store);
    }
}

/// Rewrites zoom and run labels from session state.
pub fn refresh(store: &Store) {
    let zoom = store.camera().zoom;
    let run_label = if graph_running(store) { "run" } else { "pause" };
    store.put(
        addresses::TOOLBAR_ZOOM_KEY,
        &encode_space(&text_field(0.02, &format!("zoom {zoom:.0}"))),
    );
    store.put(
        addresses::TOOLBAR_RUN_KEY,
        &encode_space(&run_field(run_label)),
    );
}

fn run_field(run: &str) -> SpaceRecord {
    SpaceRecord {
        across: [0.08, 0.08, 0.0],
        down: [0.04, 0.04, 0.0],
        style: "plain".into(),
        detail_override: None,
        hosts_space: false,
        accepts: true,
        origin: [0.34, 0.02],
        primitive: "text".into(),
        link: None,
        text: run.into(),
    }
}

fn text_field(origin_y: f64, run: &str) -> SpaceRecord {
    SpaceRecord {
        across: [0.0, 0.0, 0.0],
        down: [0.0, 0.025, 0.0],
        style: "plain".into(),
        detail_override: None,
        hosts_space: false,
        accepts: false,
        origin: [0.02, origin_y],
        primitive: "text".into(),
        link: None,
        text: run.into(),
    }
}
