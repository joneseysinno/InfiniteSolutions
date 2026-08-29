//! Drive the interpreted behaviour. Names no layer crate.

use crate::editor::addresses;
use crate::editor::palette;
use crate::editor::toolbar;
use crate::editor::wire;
use crate::facade::Store;

/// Registers the behaviour plan and the derived artifacts.
pub fn bind(store: &Store) {
    store.bind_plan(
        addresses::BEHAVIOUR_ROOT_KEY,
        addresses::BEHAVIOUR_END_KEY,
        addresses::BEHAVIOUR_ROOT_KEY,
    );
    store.bind_graph(addresses::GRAPH_ROOT_KEY);
    store.bind_styles(addresses::STYLE_ROOT_KEY, addresses::STYLE_END_KEY);
    store.bind_background(addresses::CANVAS_KEY);
}

fn point_xy(payload: &[u8]) -> Option<(f64, f64)> {
    if payload.len() < 16 {
        return None;
    }
    Some((
        f64::from_le_bytes(payload[0..8].try_into().ok()?),
        f64::from_le_bytes(payload[8..16].try_into().ok()?),
    ))
}

fn wire_mismatch(store: &Store) -> bool {
    store
        .pending_at(addresses::WIRE_MISMATCH_KEY)
        .is_some_and(|b| b.first().copied().unwrap_or(0) != 0)
}

/// Fills unbound ports from pending input and runs the linked plan.
pub fn run(store: &Store) {
    let button = store
        .pending_at(addresses::POINTER_BUTTON.as_bytes())
        .unwrap_or_else(|| vec![0]);
    let pos = store.pending_at(addresses::POINTER_POSITION.as_bytes());
    let wire_drag = store.pending_at(addresses::WIRE_FROM_KEY).is_some();

    if button.first().copied().unwrap_or(0) != 0 {
        if let Some(p) = &pos {
            if let Some((x, y)) = point_xy(p) {
                if wire::mode_active(store) && store.pending_at(addresses::WIRE_FROM_KEY).is_none() {
                    if let Some(hit) = store.probe_at(x, y) {
                        if wire::is_endpoint(store, &hit) {
                            let _ = wire::begin(store, &hit);
                        }
                    }
                } else if let Some(hit) = palette::hit_at(store, x, y) {
                    store.amend(addresses::PALETTE_FROM_KEY, &hit);
                }
            }
            if !wire_drag
                && !wire::mode_active(store)
                && store.pending_at(addresses::WIRE_FROM_KEY).is_none()
                && store.pending_at(addresses::PALETTE_FROM_KEY).is_none()
                && store.pending_at(addresses::DRAG_FROM_KEY).is_none()
                && pos.as_ref().is_none_or(|p| {
                    point_xy(p).is_none_or(|(x, y)| toolbar::hit_at(store, x, y).is_none())
                })
            {
                store.amend(addresses::DRAG_FROM_KEY, p);
            }
        }
        if wire_drag {
            if let Some(p) = &pos {
                if let Some((x, y)) = point_xy(p) {
                    if let Some(hit) = store.probe_at(x, y) {
                        wire::update(store, &hit, wire_mismatch(store));
                    }
                }
            }
        }
    } else {
        store.discard_at(addresses::DRAG_FROM_KEY);
        if store.pending_at(addresses::PALETTE_FROM_KEY).is_some() {
            if let Some(p) = &pos {
                if let Some((x, y)) = point_xy(p) {
                    let hit = store.probe_at(x, y).unwrap_or_default();
                    if !palette::is_palette_item(&hit) {
                        let _ = palette::prepare_drop(store, x, y, &hit);
                    }
                }
            }
        }
        if store.pending_at(addresses::WIRE_FROM_KEY).is_some() {
            if let Some(p) = &pos {
                if let Some((x, y)) = point_xy(p) {
                    if let Some(hit) = store.probe_at(x, y) {
                        wire::finish(store, &hit, wire_mismatch(store));
                    }
                }
            }
        }
        if store.pending_at(addresses::RELEASE_PULSE_KEY).is_some() {
            if let Some(p) = &pos {
                if let Some((x, y)) = point_xy(p) {
                    toolbar::activate(store, x, y);
                }
            }
        }
    }
    let palette_drag = store.pending_at(addresses::PALETTE_FROM_KEY).is_some();
    if let Some(p) = &pos {
        store.write_slot(addresses::BEHAVIOUR_PROBE_KEY, "at", p, "point");
        store.write_slot(addresses::BEHAVIOUR_OFFSET_KEY, "to", p, "point");
    }
    if let Some(from) = store.pending_at(addresses::DRAG_FROM_KEY) {
        store.write_slot(addresses::BEHAVIOUR_OFFSET_KEY, "from", &from, "point");
    }
    if !palette_drag
        && !wire_drag
        && !wire::mode_active(store)
        && store.pending_at(addresses::WIRE_FROM_KEY).is_none()
        && store.pending_at(addresses::PALETTE_FROM_KEY).is_none()
    {
        store.write_slot(addresses::BEHAVIOUR_GATE_KEY, "on", &button, "flag");
    }
    if let Some(pulse) = store.pending_at(addresses::RELEASE_PULSE_KEY) {
        store.write_slot(addresses::BEHAVIOUR_SELECT_GATE_KEY, "on", &pulse, "flag");
    }
    if let Some(sel) = store.selection() {
        store.write_slot(addresses::BEHAVIOUR_EDIT_READ_KEY, "addr", &sel, "address");
        store.write_slot(addresses::BEHAVIOUR_EDIT_AMEND_KEY, "addr", &sel, "address");
        store.write_slot(addresses::BEHAVIOUR_EDIT_COMMIT_KEY, "addr", &sel, "address");
    }
    if let Some(origin) = store.pending_at(addresses::EDIT_ORIGIN_KEY) {
        store.write_slot(addresses::BEHAVIOUR_SET_ORIGIN_KEY, "origin", &origin, "point");
    }
    if let Some(pulse) = store.pending_at(addresses::EDIT_COMMIT_KEY) {
        store.write_slot(addresses::BEHAVIOUR_EDIT_GATE_KEY, "on", &pulse, "flag");
    }
    if let Some(from) = store.pending_at(addresses::PALETTE_FROM_KEY) {
        store.write_slot(addresses::BEHAVIOUR_PLACE_READ_KEY, "addr", &from, "address");
    }
    if let Some(origin) = store.pending_at(addresses::PLACE_ORIGIN_KEY) {
        store.write_slot(
            addresses::BEHAVIOUR_PLACE_SET_ORIGIN_KEY,
            "origin",
            &origin,
            "point",
        );
    }
    if let Some(pulse) = store.pending_at(addresses::PLACE_COMMIT_KEY) {
        store.write_slot(addresses::BEHAVIOUR_PLACE_GATE_KEY, "on", &pulse, "flag");
    }
    if let Some(addr) = store.pending_at(addresses::PLACE_ADDR_KEY) {
        store.write_slot(addresses::BEHAVIOUR_PLACE_AMEND_KEY, "addr", &addr, "address");
        store.write_slot(addresses::BEHAVIOUR_PLACE_COMMIT_KEY, "addr", &addr, "address");
    }
    if let Some(from) = store.pending_at(addresses::WIRE_FROM_KEY) {
        store.write_slot(addresses::BEHAVIOUR_ENCODE_WIRE_KEY, "from", &from, "address");
    }
    if let Some(to) = store.pending_at(addresses::WIRE_TO_KEY) {
        store.write_slot(addresses::BEHAVIOUR_ENCODE_WIRE_KEY, "to", &to, "address");
    }
    if let Some(pulse) = store.pending_at(addresses::WIRE_COMMIT_KEY) {
        store.write_slot(addresses::BEHAVIOUR_WIRE_GATE_KEY, "on", &pulse, "flag");
    }
    if let Some(addr) = store.pending_at(addresses::WIRE_ADDR_KEY) {
        store.write_slot(addresses::BEHAVIOUR_WIRE_AMEND_KEY, "addr", &addr, "address");
        store.write_slot(addresses::BEHAVIOUR_WIRE_COMMIT_KEY, "addr", &addr, "address");
    }
    store.write_slot(
        addresses::BEHAVIOUR_SELECT_AMEND_KEY,
        "addr",
        addresses::SELECT_KEY,
        "address",
    );
    store.write_slot(
        addresses::BEHAVIOUR_SELECT_COMMIT_KEY,
        "addr",
        addresses::SELECT_KEY,
        "address",
    );
    store.run_linked();
    store.discard_at(addresses::RELEASE_PULSE_KEY);
    store.discard_at(addresses::EDIT_COMMIT_KEY);
    store.discard_at(addresses::PLACE_COMMIT_KEY);
    store.discard_at(addresses::WIRE_COMMIT_KEY);
    if button.first().copied().unwrap_or(0) == 0 {
        store.discard_at(addresses::WIRE_MISMATCH_KEY);
        store.discard_at(addresses::PALETTE_FROM_KEY);
        store.discard_at(addresses::WIRE_FROM_KEY);
        store.discard_at(addresses::WIRE_TO_KEY);
    }
}
