//! Drive the interpreted behaviour. Names no layer crate.

use crate::editor::addresses;
use crate::facade::Store;

/// Registers the behaviour plan and the derived artifacts.
pub fn bind(store: &Store) {
    store.bind_plan(
        addresses::BEHAVIOUR_ROOT_KEY,
        addresses::BEHAVIOUR_END_KEY,
        addresses::BEHAVIOUR_ROOT_KEY,
    );
    store.bind_graph(addresses::GRAPH_ROOT_KEY);
    // D44 and E10.2: the app owns its addresses (D34), so the app hands the facade
    // the style table and the space whose fill is the background. The facade never
    // names an editor address, and the background is authored rather than a constant.
    store.bind_styles(addresses::STYLE_ROOT_KEY, addresses::STYLE_END_KEY);
    store.bind_background(addresses::CANVAS_KEY);
}

/// Fills unbound ports from pending input and runs the linked plan.
pub fn run(store: &Store) {
    let button = store
        .pending_at(addresses::POINTER_BUTTON.as_bytes())
        .unwrap_or_else(|| vec![0]);
    let pos = store.pending_at(addresses::POINTER_POSITION.as_bytes());
    if button.first().copied().unwrap_or(0) != 0 {
        if store.pending_at(addresses::DRAG_FROM_KEY).is_none() {
            if let Some(p) = &pos {
                store.amend(addresses::DRAG_FROM_KEY, p);
            }
        }
    } else {
        store.discard_at(addresses::DRAG_FROM_KEY);
    }
    if let Some(p) = &pos {
        store.write_slot(addresses::BEHAVIOUR_PROBE_KEY, "at", p, "point");
        store.write_slot(addresses::BEHAVIOUR_OFFSET_KEY, "to", p, "point");
    }
    if let Some(from) = store.pending_at(addresses::DRAG_FROM_KEY) {
        store.write_slot(addresses::BEHAVIOUR_OFFSET_KEY, "from", &from, "point");
    }
    store.write_slot(addresses::BEHAVIOUR_GATE_KEY, "on", &button, "flag");
    if let Some(pulse) = store.pending_at(addresses::RELEASE_PULSE_KEY) {
        store.write_slot(addresses::BEHAVIOUR_SELECT_GATE_KEY, "on", &pulse, "flag");
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
}
