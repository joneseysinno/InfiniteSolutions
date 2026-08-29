//! E13.7 — a counter authored by pointer: place, wire, click, persist.

use infinite_solutions::editor;
use infinite_solutions::editor::addresses;
use infinite_solutions::editor::mint;
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

fn drain(store: &facade::Store) {
    for _ in 0..32 {
        if store.committed_len() == 0 {
            break;
        }
        store.tick();
    }
    store.sync().expect("sync");
}

fn center(store: &facade::Store, key: &[u8]) -> (f64, f64) {
    let placement = store.place_now();
    let target = placement
        .placed
        .iter()
        .find(|p| p.at.as_bytes() == key)
        .expect("target is placed");
    (
        (target.rect.min.x + target.rect.max.x) * 0.5,
        (target.rect.min.y + target.rect.max.y) * 0.5,
    )
}

fn drag_palette_to(store: &facade::Store, template: &[u8], drop_x: f64, drop_y: f64) -> Vec<u8> {
    let minted = mint::next_child(store, addresses::CANVAS_KEY).expect("mint child");
    let origin = mint::local_origin(store, addresses::CANVAS_KEY, drop_x, drop_y).expect("origin");
    store.amend(addresses::PALETTE_FROM_KEY, template);
    store.amend(addresses::PLACE_ADDR_KEY, &minted);
    store.amend(addresses::PLACE_ORIGIN_KEY, &origin);
    store.amend(addresses::PLACE_COMMIT_KEY, &[1]);
    editor::run(store);
    store.commit_at(&minted);
    drain(store);
    minted
}

fn wire(store: &facade::Store, from: &[u8], to: &[u8]) {
    let (fx, fy) = center(store, from);
    let (tx, ty) = center(store, to);
    assert_eq!(
        store.probe_at(fx, fy).as_deref(),
        Some(from),
        "wire must start from the bump block"
    );
    store.amend(addresses::WIRE_MODE_KEY, &[1]);
    store.amend(addresses::POINTER_POSITION.as_bytes(), &point(fx, fy));
    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[1]);
    editor::run(store);
    store.amend(addresses::POINTER_POSITION.as_bytes(), &point(tx, ty));
    editor::run(store);
    assert_eq!(
        store.probe_at(tx, ty).as_deref(),
        Some(to),
        "wire must finish on the total block"
    );
    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[0]);
    store.amend(addresses::RELEASE_PULSE_KEY, &[1]);
    editor::run(store);
    if let Some(addr) = store.pending_at(addresses::WIRE_ADDR_KEY) {
        store.commit_at(&addr);
    }
    drain(store);
    store.discard_at(addresses::WIRE_MODE_KEY);
}

fn click(store: &facade::Store, key: &[u8]) {
    let (x, y) = center(store, key);
    store.amend(addresses::POINTER_POSITION.as_bytes(), &point(x, y));
    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[1]);
    editor::run(store);
    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[0]);
    store.amend(addresses::RELEASE_PULSE_KEY, &[1]);
    editor::run(store);
    drain(store);
}

fn total_text(store: &facade::Store, key: &[u8]) -> String {
    decode_space(&store.stored_at(key).expect("total stored"))
        .expect("IS1")
        .text
}

fn canvas_drop(store: &facade::Store, x_frac: f64, y_frac: f64) -> (f64, f64) {
    let placement = store.place_now();
    let canvas = placement
        .placed
        .iter()
        .find(|p| p.at.as_bytes() == addresses::CANVAS_KEY)
        .expect("canvas");
    let x = canvas.rect.min.x + (canvas.rect.max.x - canvas.rect.min.x) * x_frac;
    let y = canvas.rect.min.y + (canvas.rect.max.y - canvas.rect.min.y) * y_frac;
    (x, y)
}

fn build_counter(store: &facade::Store) -> (Vec<u8>, Vec<u8>) {
    let (total_x, total_y) = canvas_drop(store, 0.25, 0.40);
    let (bump_x, bump_y) = canvas_drop(store, 0.70, 0.40);
    let total = drag_palette_to(store, addresses::PALETTE_TOTAL_KEY, total_x, total_y);
    let bump = drag_palette_to(store, addresses::PALETTE_BUMP_KEY, bump_x, bump_y);
    wire(store, &bump, &total);
    assert!(
        store.has(addresses::APP_ROOT_KEY),
        "wiring bump to total must install the app graph"
    );
    (bump, total)
}

#[test]
fn a_counter_authored_by_pointer_increments_and_persists() {
    let (dir, store) = seeded();
    let (bump, total) = build_counter(&store);

    for expected in 1..=3 {
        click(&store, &bump);
        assert_eq!(
            total_text(&store, &total),
            expected.to_string(),
            "click {expected} must increment the total"
        );
    }

    drop(store);
    let store = facade::open(dir.path()).expect("reopen");
    editor::bind(&store);
    store.set_surface(0.0, 0.0, 800.0, 600.0, 1.0);
    assert_eq!(
        total_text(&store, &total),
        "3",
        "the counter total must survive restart"
    );
}
