//! Drive the interpreted behaviour. Names no layer crate.

use crate::editor::addresses;
use crate::editor::app;
use crate::editor::palette;
use crate::editor::toolbar;
use crate::editor::wire;
use crate::facade::Store;

/// Registers the behaviour plan and the derived artifacts.
pub fn bind(store: &Store) {
    store.bind_plan(
        addresses::BEHAVIOUR_ROOT_KEY,
        addresses::behaviour_end_key(),
        addresses::BEHAVIOUR_ROOT_KEY,
    );
    store.bind_graph(addresses::GRAPH_ROOT_KEY);
    store.bind_styles(addresses::STYLE_ROOT_KEY, addresses::style_end_key());
    store.bind_background(addresses::canvas_key());
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
        .pending_at(addresses::wire_mismatch_key())
        .is_some_and(|b| b.first().copied().unwrap_or(0) != 0)
}

/// Fills unbound ports from pending input and runs the linked plan.
pub fn run(store: &Store) {
    let button = store
        .pending_at(addresses::POINTER_BUTTON.as_bytes())
        .unwrap_or_else(|| vec![0]);
    let pos = store.pending_at(addresses::POINTER_POSITION.as_bytes());
    let wire_drag = store.pending_at(addresses::wire_from_key()).is_some();

    if button.first().copied().unwrap_or(0) != 0 {
        if let Some(p) = &pos {
            if let Some((x, y)) = point_xy(p) {
                if wire::mode_active(store) && store.pending_at(addresses::wire_from_key()).is_none() {
                    if let Some(hit) = store.probe_at(x, y) {
                        if wire::is_endpoint(store, &hit) {
                            let _ = wire::begin(store, &hit);
                        }
                    }
                } else if store.pending_at(addresses::palette_from_key()).is_none() {
                    if let Some(hit) = palette::hit_at(store, x, y) {
                        store.amend(addresses::palette_from_key(), &hit);
                    }
                }
            }
            if !wire_drag
                && !wire::mode_active(store)
                && store.pending_at(addresses::wire_from_key()).is_none()
                && store.pending_at(addresses::palette_from_key()).is_none()
                && store.pending_at(addresses::drag_from_key()).is_none()
                && pos.as_ref().is_none_or(|p| {
                    point_xy(p).is_none_or(|(x, y)| toolbar::hit_at(store, x, y).is_none())
                })
            {
                store.amend(addresses::drag_from_key(), p);
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
        store.discard_at(addresses::drag_from_key());
        if store.pending_at(addresses::palette_from_key()).is_some() {
            if let Some(p) = &pos {
                if let Some((x, y)) = point_xy(p) {
                    let hit = store.probe_at(x, y).unwrap_or_default();
                    if !palette::is_palette_item(&hit) {
                        let _ = palette::prepare_drop(store, x, y, &hit);
                    }
                }
            }
        }
        if store.pending_at(addresses::wire_from_key()).is_some() {
            if let Some(p) = &pos {
                if let Some((x, y)) = point_xy(p) {
                    if let Some(hit) = store.probe_at(x, y) {
                        wire::finish(store, &hit, wire_mismatch(store));
                    }
                }
            }
        }
        if store.pending_at(addresses::release_pulse_key()).is_some() {
            if let Some(p) = &pos {
                if let Some((x, y)) = point_xy(p) {
                    toolbar::activate(store, x, y);
                }
            }
        }
    }
    let palette_drag = store.pending_at(addresses::palette_from_key()).is_some();
    if let Some(p) = &pos {
        store.write_slot(addresses::behaviour_probe_key(), "at", p, "point");
        store.write_slot(addresses::behaviour_offset_key(), "to", p, "point");
    }
    if let Some(from) = store.pending_at(addresses::drag_from_key()) {
        store.write_slot(addresses::behaviour_offset_key(), "from", &from, "point");
    }
    if !palette_drag
        && !wire_drag
        && !wire::mode_active(store)
        && store.pending_at(addresses::wire_from_key()).is_none()
        && store.pending_at(addresses::palette_from_key()).is_none()
    {
        store.write_slot(addresses::behaviour_gate_key(), "on", &button, "flag");
    }
    if let Some(pulse) = store.pending_at(addresses::release_pulse_key()) {
        store.write_slot(addresses::behaviour_select_gate_key(), "on", &pulse, "flag");
    }
    if let Some(sel) = store.selection() {
        store.write_slot(addresses::behaviour_edit_read_key(), "addr", &sel, "address");
        store.write_slot(addresses::behaviour_edit_amend_key(), "addr", &sel, "address");
        store.write_slot(addresses::behaviour_edit_commit_key(), "addr", &sel, "address");
    }
    if let Some(origin) = store.pending_at(addresses::edit_origin_key()) {
        store.write_slot(addresses::behaviour_set_origin_key(), "origin", &origin, "point");
    }
    if let Some(pulse) = store.pending_at(addresses::edit_commit_key()) {
        store.write_slot(addresses::behaviour_edit_gate_key(), "on", &pulse, "flag");
    }
    if let Some(from) = store.pending_at(addresses::palette_from_key()) {
        store.write_slot(addresses::behaviour_place_read_key(), "addr", &from, "address");
    }
    if let Some(origin) = store.pending_at(addresses::place_origin_key()) {
        store.write_slot(
            addresses::behaviour_place_set_origin_key(),
            "origin",
            &origin,
            "point",
        );
    }
    if let Some(pulse) = store.pending_at(addresses::place_commit_key()) {
        store.write_slot(addresses::behaviour_place_gate_key(), "on", &pulse, "flag");
    }
    if let Some(addr) = store.pending_at(addresses::place_addr_key()) {
        store.write_slot(addresses::behaviour_place_amend_key(), "addr", &addr, "address");
        store.write_slot(addresses::behaviour_place_commit_key(), "addr", &addr, "address");
    }
    if let Some(from) = store.pending_at(addresses::wire_from_key()) {
        store.write_slot(addresses::behaviour_encode_wire_key(), "from", &from, "address");
    }
    if let Some(to) = store.pending_at(addresses::wire_to_key()) {
        store.write_slot(addresses::behaviour_encode_wire_key(), "to", &to, "address");
    }
    if let Some(pulse) = store.pending_at(addresses::wire_commit_key()) {
        store.write_slot(addresses::behaviour_wire_gate_key(), "on", &pulse, "flag");
    }
    if let Some(addr) = store.pending_at(addresses::wire_addr_key()) {
        store.write_slot(addresses::behaviour_wire_amend_key(), "addr", &addr, "address");
        store.write_slot(addresses::behaviour_wire_commit_key(), "addr", &addr, "address");
    }
    store.write_slot(
        addresses::behaviour_select_amend_key(),
        "addr",
        addresses::select_key(),
        "address",
    );
    store.write_slot(
        addresses::behaviour_select_commit_key(),
        "addr",
        addresses::select_key(),
        "address",
    );
    store.run_linked();
    let release = store.pending_at(addresses::release_pulse_key()).is_some();
    if release {
        app::try_run(store);
    }
    store.discard_at(addresses::release_pulse_key());
    store.discard_at(addresses::edit_commit_key());
    store.discard_at(addresses::place_commit_key());
    store.discard_at(addresses::wire_commit_key());
    if button.first().copied().unwrap_or(0) == 0 {
        store.discard_at(addresses::wire_mismatch_key());
        store.discard_at(addresses::palette_from_key());
        store.discard_at(addresses::wire_from_key());
        store.discard_at(addresses::wire_to_key());
    }
}
