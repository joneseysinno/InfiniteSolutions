//! E13.6 — the toolbar survives §4's test: undo/redo, zoom readout, run/pause.
//!
//! Three affordances as authored spaces — not a widget layer (`EDITOR.md` §2.1).

use infinite_solutions::editor;
use infinite_solutions::editor::addresses;
use infinite_solutions::editor::toolbar;
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
    editor::refresh_toolbar(&store);
    (dir, store)
}

fn field_text(store: &facade::Store, key: &[u8]) -> String {
    decode_space(&store.stored_at(key).expect("toolbar field stored"))
        .expect("IS1")
        .text
}

fn click_toolbar(store: &facade::Store, key: &[u8], x_fraction: f64) {
    let placement = store.place_now();
    let target = placement
        .placed
        .iter()
        .find(|p| p.at.as_bytes() == key)
        .expect("toolbar affordance is placed");
    let x = target.rect.min.x + (target.rect.max.x - target.rect.min.x) * x_fraction;
    let y = (target.rect.min.y + target.rect.max.y) * 0.5;
    store.amend(addresses::POINTER_POSITION.as_bytes(), &point(x, y));
    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[1]);
    editor::run(store);
    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[0]);
    store.amend(addresses::release_pulse_key(), &[1]);
    editor::run(store);
    editor::refresh_toolbar(store);
}

fn drag_and_commit(store: &facade::Store) {
    let placement = store.place_now();
    let target = placement
        .placed
        .iter()
        .find(|p| p.at.as_bytes() == addresses::node_a_key())
        .expect("node A");
    let mid_x = (target.rect.min.x + target.rect.max.x) * 0.5;
    let mid_y = (target.rect.min.y + target.rect.max.y) * 0.5;
    store.amend(
        addresses::drag_from_key(),
        &point(mid_x - 10.0, mid_y),
    );
    store.amend(addresses::POINTER_POSITION.as_bytes(), &point(mid_x, mid_y));
    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[1]);
    editor::run(store);
    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[0]);
    editor::run(store);
    for key in [
        addresses::canvas_key(),
        addresses::node_a_key(),
        addresses::node_b_key(),
    ] {
        store.commit_at(key);
    }
    for _ in 0..32 {
        if store.committed_len() == 0 {
            break;
        }
        store.tick();
    }
    store.sync().expect("sync");
}

#[test]
fn clicking_undo_on_the_toolbar_restores_a_committed_drag() {
    let (_dir, store) = seeded();
    let before = decode_space(&store.stored_at(addresses::node_a_key()).unwrap())
        .unwrap()
        .origin;
    drag_and_commit(&store);
    assert_ne!(
        decode_space(&store.stored_at(addresses::node_a_key()).unwrap())
            .unwrap()
            .origin,
        before
    );

    click_toolbar(&store, addresses::toolbar_history_key(), 0.25);
    assert_eq!(
        decode_space(&store.stored_at(addresses::node_a_key()).unwrap())
            .unwrap()
            .origin,
        before,
        "toolbar undo must restore the pre-drag origin"
    );
}

#[test]
fn the_zoom_readout_follows_the_session_camera() {
    let (_dir, store) = seeded();
    let before = field_text(&store, addresses::toolbar_zoom_key());
    store.zoom_by(2.0);
    editor::refresh_toolbar(&store);
    let after = field_text(&store, addresses::toolbar_zoom_key());
    assert_ne!(before, after, "zoom label must change after zoom_by");
    assert!(
        after.contains(&format!("{:.0}", store.camera().zoom)),
        "zoom label must show the current magnification"
    );
}

#[test]
fn pausing_from_the_toolbar_stops_the_tick_loop() {
    let (_dir, store) = seeded();
    drag_and_commit(&store);
    store.amend(addresses::node_a_key(), b"pending-gesture");
    store.commit_at(addresses::node_a_key());
    assert!(store.committed_len() > 0, "setup: a commit is waiting to drain");

    click_toolbar(&store, addresses::toolbar_run_key(), 0.5);
    assert!(
        field_text(&store, addresses::toolbar_run_key()).contains("pause"),
        "run affordance must show pause while halted"
    );
    assert!(!toolbar::graph_running(&store));
    if toolbar::graph_running(&store) {
        store.tick();
    }
    assert_eq!(
        store.committed_len(),
        1,
        "while paused, the drive path must not drain commits"
    );

    click_toolbar(&store, addresses::toolbar_run_key(), 0.5);
    for _ in 0..32 {
        if store.committed_len() == 0 {
            break;
        }
        if toolbar::graph_running(&store) {
            store.tick();
        }
    }
    assert_eq!(
        store.committed_len(),
        0,
        "resuming run must drain the waiting commit"
    );
}
