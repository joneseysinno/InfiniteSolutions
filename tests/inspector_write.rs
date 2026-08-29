//! E13.3 — the property inspector writes through the behaviour composition.

use infinite_solutions::editor;
use infinite_solutions::editor::addresses;
use infinite_solutions::facade::{self, decode_space};

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

fn drain(store: &facade::Store) {
    for _ in 0..32 {
        if store.committed_len() == 0 {
            break;
        }
        store.tick();
    }
    store.sync().expect("sync");
}

fn stored_origin(store: &facade::Store, key: &[u8]) -> [f64; 2] {
    decode_space(&store.stored_at(key).expect("stored space"))
        .expect("IS1")
        .origin
}

#[test]
fn editing_origin_follows_the_canvas_in_the_same_frame() {
    let (_dir, store) = seeded();
    click(&store, addresses::node_a_key());

    editor::apply_origin(&store, 0.1, 0.1);
    editor::run(&store);

    let view = store.selection_view().expect("selection view after edit run");
    assert_eq!(
        view.origin,
        [0.1, 0.1],
        "the scene port must overlay the pending amend before commit"
    );

    store.commit_at(addresses::node_a_key());
    drain(&store);

    assert_eq!(
        stored_origin(&store, addresses::node_a_key()),
        [0.1, 0.1],
        "the committed record carries the edited origin"
    );
}

#[test]
fn undoing_an_inspector_edit_restores_the_previous_origin() {
    let (_dir, store) = seeded();
    click(&store, addresses::node_a_key());
    let before = stored_origin(&store, addresses::node_a_key());

    editor::apply_origin(&store, 0.1, 0.1);
    editor::run(&store);
    store.commit_at(addresses::node_a_key());
    drain(&store);

    let touched = store.undo();
    assert_eq!(
        touched.as_deref(),
        Some(addresses::node_a_key()),
        "undo must report the edited address"
    );
    assert_eq!(
        stored_origin(&store, addresses::node_a_key()),
        before,
        "undo must restore the pre-edit origin"
    );
}

#[test]
fn the_inspector_never_amends_the_selected_node_directly() {
    let source = include_str!("../src/editor/inspector.rs");
    for line in source.lines() {
        let t = line.trim();
        if t.starts_with("///") || t.starts_with("//") {
            continue;
        }
        assert!(
            !t.contains("node_a_key") && !t.contains("node_b_key"),
            "the inspector must not name authored node keys: {t}"
        );
        if t.contains("store.amend(") {
            assert!(
                t.contains("edit_origin_key") || t.contains("edit_commit_key"),
                "inspector amend must target gesture addresses only: {t}"
            );
        }
    }
}
