//! E7 — the editor edits itself and it persists.
//!
//! Drag a node belonging to the editor's own screen while the editor is running.
//! The change persists. Restart. It is still there. Nothing was recompiled.

use infinite_compositor::binding::ports::Backends as BackendsPort;
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

fn screen_origins(store: &facade::Store) -> Vec<([u8; 4], [f64; 2])> {
    let keys = [
        addresses::CANVAS_KEY,
        addresses::NODE_A_KEY,
        addresses::NODE_B_KEY,
    ];
    keys.iter()
        .map(|k| {
            let mut id = [0u8; 4];
            id.copy_from_slice(k);
            (id, stored_origin(store, k))
        })
        .collect()
}

fn drag(store: &facade::Store, delta: (f64, f64)) {
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
}

fn persist(store: &facade::Store) {
    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[0]);
    editor::run(store);
    for key in [
        addresses::CANVAS_KEY,
        addresses::NODE_A_KEY,
        addresses::NODE_B_KEY,
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
fn editing_the_editors_own_screen_persists() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().to_path_buf();

    let before;
    let moved;
    {
        let store = facade::open(&path).expect("open");
        editor::seed(|k| store.has(k), |k, v| store.put(k, v));
        editor::bind(&store);
        store.set_surface(0.0, 0.0, 800.0, 600.0, 1.0);
        let _ = store.place_now();

        before = screen_origins(&store);
        drag(&store, (10.0, 0.0));
        persist(&store);
        moved = screen_origins(&store);
        assert_ne!(
            before, moved,
            "a genesis node must move while the editor is running"
        );
        assert!(
            BackendsPort::backend(&store.backends(), "native").is_none(),
            "self-edit does not generate Rust; tier 0 is not a toolchain"
        );
        drop(store);
    }

    let store = facade::open(&path).expect("restart");
    editor::seed(|k| store.has(k), |k, v| store.put(k, v));
    let after = screen_origins(&store);
    assert_eq!(
        moved, after,
        "the edited screen survives restart; genesis does not overwrite it"
    );
    assert_ne!(
        before, after,
        "the restart did not restore the unedited genesis screen"
    );
    let _ = store.place_now();
    let _ = store.tick();
}
