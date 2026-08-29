//! E6 — the drag is interpreted, and provenance is exact.
//!
//! Verified by deleting the composition and observing that dragging stops while
//! the window still runs (R23). Provenance recovers the exact declared input set.
//! An input change at revision N yields exactly the downstream address set.

use infinite_compositor::core::PortRef;
use infinite_solutions::editor;
use infinite_solutions::editor::addresses;
use infinite_solutions::facade::{self, decode_space, compositor_addr};

fn point(x: f64, y: f64) -> Vec<u8> {
    let mut p = Vec::with_capacity(16);
    p.extend_from_slice(&x.to_le_bytes());
    p.extend_from_slice(&y.to_le_bytes());
    p
}

fn live_origin(store: &facade::Store, key: &[u8]) -> [f64; 2] {
    if let Some(payload) = store.pending_at(key) {
        if let Some(space) = decode_space(&payload) {
            return space.origin;
        }
    }
    decode_space(&store.stored_at(key).expect("stored space"))
        .expect("IS1")
        .origin
}

fn slot(block: &[u8], port: &str) -> Vec<u8> {
    PortRef {
        block: compositor_addr(block),
        port: port.into(),
    }
    .slot()
    .as_bytes()
    .to_vec()
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

fn drag(store: &facade::Store, delta: (f64, f64)) {
    let placement = store.place_now();
    let target = placement
        .placed
        .iter()
        .find(|p| p.at.as_bytes() == addresses::node_a_key())
        .or_else(|| placement.placed.iter().find(|p| p.accepts))
        .expect("a space to drag");
    let mid_x = (target.rect.min.x + target.rect.max.x) * 0.5;
    let mid_y = (target.rect.min.y + target.rect.max.y) * 0.5;
    store.amend(
        addresses::drag_from_key(),
        &point(mid_x - delta.0, mid_y - delta.1),
    );
    store.amend(addresses::POINTER_POSITION.as_bytes(), &point(mid_x, mid_y));
    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[1]);
    editor::run(store);
}

fn any_origin(store: &facade::Store) -> Vec<(Vec<u8>, [f64; 2])> {
    let keys = [
        addresses::canvas_key(),
        addresses::node_a_key(),
        addresses::node_b_key(),
    ];
    keys.iter()
        .map(|k| (k.to_vec(), live_origin(store, k)))
        .collect()
}

#[test]
fn drag_is_performed_by_the_interpreted_composition() {
    let (_dir, store) = seeded();
    assert!(
        store.artifacts_pass_discard(),
        "Placement and Plan pass the generic discard harness"
    );

    let before = any_origin(&store);
    drag(&store, (10.0, 0.0));
    let moved = any_origin(&store);
    assert_ne!(
        before, moved,
        "the interpreted composition must move the space"
    );

    let out = slot(addresses::behaviour_offset_key(), "out");
    let fn_in = slot(addresses::behaviour_offset_key(), "fn");
    let val_in = slot(addresses::behaviour_offset_key(), "val");
    let aux_in = slot(addresses::behaviour_offset_key(), "aux");
    let declared = store.inputs_of(&out);
    assert_eq!(
        declared,
        vec![fn_in, val_in.clone(), aux_in],
        "provenance recovers the exact declared input set"
    );
    let downstream = store.stale_downstream(&val_in);
    assert_eq!(
        downstream,
        vec![out],
        "an input yields exactly the downstream address set"
    );

    store.delete_key(addresses::BEHAVIOUR_ROOT_KEY);
    let _ = store.tick();
    drag(&store, (40.0, 0.0));
    let after_delete = any_origin(&store);
    assert_eq!(
        moved, after_delete,
        "deleting the composition stops the drag; the portal did not move it"
    );
    let report = store.tick();
    let _ = report.submitted;
}
