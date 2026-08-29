//! E20 — one Innovator screen, role-routed commit, re-seed without builders.

use infinite_solutions::editor;
use infinite_solutions::editor::addresses;
use infinite_solutions::facade;

fn point(x: f64, y: f64) -> Vec<u8> {
    let mut p = Vec::with_capacity(16);
    p.extend_from_slice(&x.to_le_bytes());
    p.extend_from_slice(&y.to_le_bytes());
    p
}

fn seeded() -> (tempfile::TempDir, facade::Store) {
    let dir = tempfile::TempDir::new().unwrap();
    let store = facade::open(dir.path()).expect("open");
    editor::seed(|k| store.has(k), |k, v| store.put(k, v));
    editor::bind(&store);
    store.set_surface(0.0, 0.0, 800.0, 600.0, 1.0);
    let _ = store.place_now();
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

fn click(store: &facade::Store, key: &[u8]) {
    let (x, y) = center(store, key);
    store.amend(addresses::POINTER_POSITION.as_bytes(), &point(x, y));
    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[1]);
    editor::run(store);
    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[0]);
    store.amend(addresses::release_pulse_key(), &[1]);
    editor::run(store);
    drain(store);
}

fn type_key(store: &facade::Store, name: &str) {
    let mut payload = name.as_bytes().to_vec();
    payload.push(1);
    store.amend(addresses::KEY.as_bytes(), &payload);
    editor::run(store);
    drain(store);
}

fn field_text(store: &facade::Store, key: &[u8]) -> String {
    String::from_utf8_lossy(&store.payload_at(key).unwrap_or_default()).into_owned()
}

fn snapshot_screen(store: &facade::Store) -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut rows = store.records(addresses::innovator_key(), addresses::style_end_key());
    rows.retain(|(k, _)| k.starts_with(addresses::innovator_key()));
    rows.extend(store.records(addresses::field_row_def_key(), addresses::graph_end_key()));
    rows
}

#[test]
fn innovator_screen_is_store_data_and_reseeds_without_builders() {
    let (_dir, store) = seeded();
    for (name, key) in [
        ("panel", addresses::panel_def_key()),
        ("section_header", addresses::section_header_def_key()),
        ("field_row", addresses::field_row_def_key()),
        ("action_bar", addresses::action_bar_def_key()),
    ] {
        assert!(store.has(key), "{name} definition must live in the store");
    }

    let placement = store.place_now();
    for key in [
        addresses::innovator_header_key(),
        addresses::innovator_field_a_key(),
        addresses::innovator_field_b_key(),
        addresses::innovator_action_key(),
    ] {
        assert!(
            placement.placed.iter().any(|p| p.at.as_bytes() == key),
            "screen must place {key:02x?}"
        );
    }

    click(&store, addresses::innovator_field_a_key());
    type_key(&store, "KeyZ");
    click(&store, addresses::innovator_action_key());
    assert_eq!(
        field_text(&store, addresses::innovator_field_a_key()),
        "heightz",
        "commit is role-routed through the interpreted composition"
    );

    let rows = snapshot_screen(&store);
    assert!(!rows.is_empty());

    let dir2 = tempfile::TempDir::new().unwrap();
    let replay = facade::open(dir2.path()).expect("open replay");
    for (key, payload) in &rows {
        replay.put(key, payload);
    }
    editor::bind(&replay);
    replay.set_surface(0.0, 0.0, 800.0, 600.0, 1.0);
    let placed = replay.place_now();
    assert!(
        placed
            .placed
            .iter()
            .any(|p| p.at.as_bytes() == addresses::innovator_header_key()),
        "re-seed from the store still renders; no Rust builders"
    );
    assert_eq!(
        field_text(&replay, addresses::innovator_field_a_key()),
        "heightz"
    );
}
