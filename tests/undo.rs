//! E12 — undo and redo (D48).
//!
//! Every drag here runs through the interpreted composition, the way
//! `tests/self_edit.rs` does — not by calling `amend` and asserting on the
//! pending set directly, because undo operates on committed history, not on a
//! gesture in flight.

use infinite_solutions::editor;
use infinite_solutions::editor::addresses;
use infinite_solutions::facade::{self, decode_space};

fn point(x: f64, y: f64) -> Vec<u8> {
    let mut p = Vec::with_capacity(16);
    p.extend_from_slice(&x.to_le_bytes());
    p.extend_from_slice(&y.to_le_bytes());
    p
}

fn stored_origin(store: &facade::Store, key: &[u8]) -> [f64; 2] {
    decode_space(&store.stored_at(key).expect("stored space"))
        .expect("IS1")
        .origin
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

/// Presses, moves, and releases over node A, mirroring `tests/self_edit.rs`'s
/// `drag` + `persist` — the difference is this drains with `tick()` in between so
/// a caller can inspect committed state before doing anything else.
fn drag_and_commit(store: &facade::Store, delta: (f64, f64)) {
    let placement = store.place_now();
    let target = placement
        .placed
        .iter()
        .find(|p| p.at.as_bytes() == addresses::NODE_A_KEY)
        .or_else(|| placement.placed.iter().find(|p| p.accepts))
        .expect("a genesis space");
    let mid_x = (target.rect.min.x + target.rect.max.x) * 0.5;
    let mid_y = (target.rect.min.y + target.rect.max.y) * 0.5;
    store.amend(
        addresses::DRAG_FROM_KEY,
        &point(mid_x - delta.0, mid_y - delta.1),
    );
    store.amend(addresses::POINTER_POSITION.as_bytes(), &point(mid_x, mid_y));
    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[1]);
    editor::run(store);

    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[0]);
    editor::run(store);
    for key in [
        addresses::CANVAS_KEY,
        addresses::NODE_A_KEY,
        addresses::NODE_B_KEY,
    ] {
        store.commit_at(key);
    }
    drain(store);
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

/// E12.0 — the check that must fail on a store with no undo at all, and must
/// pass once undo writes the value read at the revision before the drag rather
/// than rewinding to it.
#[test]
fn undoing_a_drag_restores_the_previous_origin() {
    let (_dir, store) = seeded();
    let before = stored_origin(&store, addresses::NODE_A_KEY);

    drag_and_commit(&store, (10.0, 0.0));
    let moved = stored_origin(&store, addresses::NODE_A_KEY);
    assert_ne!(before, moved, "the drag must move the node");

    let touched = store.undo();
    assert_eq!(
        touched.as_deref(),
        Some(addresses::NODE_A_KEY),
        "undo must report the address it touched"
    );

    let restored = stored_origin(&store, addresses::NODE_A_KEY);
    assert_eq!(restored, before, "undo must restore the pre-drag origin");
}

/// E12.2's second clause: undo is a commit, not a rewind. `stable_revision`
/// strictly increases across an undo — a rewind would restore the value and
/// fail only this assertion.
#[test]
fn undo_increases_the_revision_rather_than_rewinding_it() {
    let (_dir, store) = seeded();
    drag_and_commit(&store, (10.0, 0.0));

    let before_undo = store.revision();
    store.undo();
    let after_undo = store.revision();
    assert!(
        after_undo > before_undo,
        "undo must land at a strictly later revision ({before_undo} -> {after_undo})"
    );
}

/// E12.1 — the commit stream is readable: empty at open, one entry after one
/// committed drag, and unchanged by a pan (D48 clause 3 — a pan never commits).
#[test]
fn committed_since_reports_commits_in_order_and_ignores_pans() {
    let (_dir, store) = seeded();
    assert!(
        store.committed_since(0).is_empty(),
        "nothing is committed at open"
    );

    drag_and_commit(&store, (10.0, 0.0));
    let after_drag = store.committed_since(0);
    assert_eq!(
        after_drag.len(),
        1,
        "one committed drag is one entry, got {after_drag:?}"
    );
    assert_eq!(after_drag[0].0, addresses::NODE_A_KEY);

    store.pan_by(40.0, 20.0);
    assert_eq!(
        store.committed_since(0),
        after_drag,
        "a pan amends the camera and never commits (D48 clause 3)"
    );
}

/// E12.3 — redo replays what undo stepped back over, and a fresh commit made
/// after an undo drops the redo tail rather than leaving it silently reachable.
#[test]
fn redo_replays_and_a_fresh_commit_drops_the_tail() {
    let (_dir, store) = seeded();
    drag_and_commit(&store, (10.0, 0.0));
    let moved = stored_origin(&store, addresses::NODE_A_KEY);

    store.undo();
    assert_ne!(stored_origin(&store, addresses::NODE_A_KEY), moved);

    let touched = store.redo();
    assert_eq!(touched.as_deref(), Some(addresses::NODE_A_KEY));
    assert_eq!(
        stored_origin(&store, addresses::NODE_A_KEY),
        moved,
        "redo must restore exactly the value undo stepped back over"
    );

    // Undo again, then make a genuinely new commit — the redo tail must be gone.
    store.undo();
    drag_and_commit(&store, (0.0, 5.0));
    assert_eq!(
        store.redo(),
        None,
        "a fresh commit after an undo must drop the redo tail, not leave it reachable"
    );
}

/// E12.4 — discard is the other verb. Escaping a drag returns the node to its
/// committed origin and adds nothing to the undo stream; a pan afterward still
/// adds nothing either.
#[test]
fn discarding_a_drag_never_enters_the_undo_stream() {
    let (_dir, store) = seeded();
    let before = stored_origin(&store, addresses::NODE_A_KEY);

    let placement = store.place_now();
    let target = placement
        .placed
        .iter()
        .find(|p| p.at.as_bytes() == addresses::NODE_A_KEY)
        .expect("node A");
    let mid_x = (target.rect.min.x + target.rect.max.x) * 0.5;
    let mid_y = (target.rect.min.y + target.rect.max.y) * 0.5;
    store.amend(addresses::DRAG_FROM_KEY, &point(mid_x - 10.0, mid_y));
    store.amend(addresses::POINTER_POSITION.as_bytes(), &point(mid_x, mid_y));
    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[1]);
    editor::run(&store);

    assert!(
        store.discard_at(addresses::NODE_A_KEY),
        "a pending, uncommitted amend at node A must be discardable"
    );
    assert!(
        store.committed_since(0).is_empty(),
        "a discard must never add an undo entry"
    );
    assert_eq!(
        stored_origin(&store, addresses::NODE_A_KEY),
        before,
        "the stored origin never changed — the amend was pending, never committed"
    );

    store.pan_by(5.0, 5.0);
    assert!(
        store.committed_since(0).is_empty(),
        "a pan must never add an undo entry either (D48 clause 3)"
    );
}

/// E12.5 — the undo stream is a registered D25 artifact, and passes the same
/// generic discard harness Placement and Plan pass, with zero per-artifact code.
#[test]
fn the_undo_stream_passes_the_generic_discard_test() {
    let (_dir, store) = seeded();
    drag_and_commit(&store, (10.0, 0.0));
    assert!(
        store.artifact_passes_discard(facade::UNDO_KEY),
        "the undo stream must survive drop-and-rebuild with identical bytes"
    );
}
