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
    }
    if let Some(p) = &pos {
        store.write_slot(addresses::BEHAVIOUR_PROBE_KEY, "at", p, "point");
        store.write_slot(addresses::BEHAVIOUR_OFFSET_KEY, "to", p, "point");
    }
    if let Some(from) = store.pending_at(addresses::DRAG_FROM_KEY) {
        store.write_slot(addresses::BEHAVIOUR_OFFSET_KEY, "from", &from, "point");
    }
    store.write_slot(addresses::BEHAVIOUR_GATE_KEY, "on", &button, "flag");
    store.run_linked();
}
