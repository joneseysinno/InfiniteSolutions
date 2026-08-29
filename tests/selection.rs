//! E13.1 — selection is authored, not a flag on `Placed`.
//!
//! A click writes [`addresses::select_key()`]; reopening the store shows the same
//! selection. `Placed` still carries no selection field (L5).

use infinite_solutions::editor;
use infinite_solutions::editor::addresses;
use infinite_solutions::facade::{self, decode_selection};

fn point(x: f64, y: f64) -> Vec<u8> {
    let mut p = Vec::with_capacity(16);
    p.extend_from_slice(&x.to_le_bytes());
    p.extend_from_slice(&y.to_le_bytes());
    p
}

fn seeded() -> (tempfile::TempDir, facade::Store) {
    let dir = tempfile::TempDir::new().unwrap();
    let store = facade::open(dir.path()).expect("open store");
    editor::seed(|k| store.has(k), |k, v| store.put(k, v));
    editor::bind(&store);
    store.set_surface(0.0, 0.0, 800.0, 600.0, 1.0);
    let _ = store.place_now();
    (dir, store)
}

fn click(store: &facade::Store, key: &[u8]) {
    let placement = store.place_now();
    let target = placement
        .placed
        .iter()
        .find(|p| p.at.as_bytes() == key)
        .expect("the target is placed");
    let mid_x = (target.rect.min.x + target.rect.max.x) * 0.5;
    let mid_y = (target.rect.min.y + target.rect.max.y) * 0.5;
    store.amend(addresses::POINTER_POSITION.as_bytes(), &point(mid_x, mid_y));
    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[1]);
    editor::run(store);
    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[0]);
    store.amend(addresses::release_pulse_key(), &[1]);
    editor::run(store);
    while store.committed_len() > 0 {
        let _ = store.tick();
    }
    let _ = store.sync();
}

#[test]
fn clicking_a_node_writes_the_selection_record() {
    let (_dir, store) = seeded();
    click(&store, addresses::node_a_key());
    let selected = store.selection().expect("selection is authored");
    assert_eq!(selected, addresses::node_a_key());
    let payload = store.stored_at(addresses::select_key()).expect("stored");
    assert_eq!(
        decode_selection(&payload),
        Some(Some(addresses::node_a_key().to_vec()))
    );
}

#[test]
fn selection_survives_restart() {
    let (dir, store) = seeded();
    click(&store, addresses::node_b_key());
    drop(store);

    let store = facade::open(dir.path()).expect("reopen");
    editor::bind(&store);
    assert_eq!(
        store.selection().as_deref(),
        Some(addresses::node_b_key()),
        "a restart replays the same selection"
    );
}

#[test]
fn selection_stops_when_the_behaviour_composition_is_removed() {
    let (_dir, store) = seeded();
    click(&store, addresses::node_a_key());
    assert_eq!(store.selection().as_deref(), Some(addresses::node_a_key()));

    store.delete_key(addresses::BEHAVIOUR_ROOT_KEY);
    let _ = store.tick();
    click(&store, addresses::node_b_key());
    assert_eq!(
        store.selection().as_deref(),
        Some(addresses::node_a_key()),
        "without the composition, a click must not change selection"
    );
}

#[test]
fn placed_still_carries_no_selection_field() {
    let field = include_str!("../crates/infinite-presenter/src/core/placed.rs");
    for line in field.lines() {
        let t = line.trim();
        if t.starts_with("///") || t.starts_with("//") {
            continue;
        }
        assert!(
            !t.contains("selected:"),
            "Placed must not grow a selection flag: {t}"
        );
    }
}
