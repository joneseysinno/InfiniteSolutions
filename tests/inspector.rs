//! E13.2 — the property inspector reads through the scene port.
//!
//! The panel is authored spaces with text primitives (§2.1). After a click, each
//! field row shows address, style, extent, origin, and depth from the address.

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
    store.amend(addresses::RELEASE_PULSE_KEY, &[1]);
    editor::run(store);
    while store.committed_len() > 0 {
        let _ = store.tick();
    }
    let _ = store.sync();
    editor::refresh_inspector(store);
}

fn field_text(store: &facade::Store, key: &[u8]) -> String {
    decode_space(&store.stored_at(key).expect("inspector field stored"))
        .expect("IS1")
        .text
}

#[test]
fn the_inspector_shows_the_selected_nodes_properties() {
    let (_dir, store) = seeded();
    click(&store, addresses::NODE_A_KEY);

    assert!(
        field_text(&store, addresses::INSPECTOR_ADDR_KEY).contains("11100000"),
        "address is shown as hex from the selection key"
    );
    assert!(
        field_text(&store, addresses::INSPECTOR_STYLE_KEY).contains("plain"),
        "style key comes from the scene port"
    );
    assert!(
        field_text(&store, addresses::INSPECTOR_ACROSS_KEY).contains("0.4"),
        "across extent comes from the scene port"
    );
    assert!(
        field_text(&store, addresses::INSPECTOR_DOWN_KEY).contains("0.4"),
        "down extent comes from the scene port"
    );
    assert!(
        field_text(&store, addresses::INSPECTOR_ORIGIN_KEY).contains("0 0"),
        "origin comes from the scene port"
    );
    assert!(
        field_text(&store, addresses::INSPECTOR_DEPTH_KEY).contains("depth 3"),
        "depth is prefix_bits / 4 from the address (D45)"
    );
}

#[test]
fn selecting_node_b_shows_its_depth_and_origin() {
    let (_dir, store) = seeded();
    click(&store, addresses::NODE_B_KEY);

    assert!(
        field_text(&store, addresses::INSPECTOR_ADDR_KEY).contains("11200000"),
        "node B's key is distinct from node A's"
    );
    assert!(
        field_text(&store, addresses::INSPECTOR_ORIGIN_KEY).contains("0.5 0.5"),
        "node B's authored origin is shown"
    );
    assert!(
        field_text(&store, addresses::INSPECTOR_DEPTH_KEY).contains("depth 3"),
        "node B sits at the same level as node A on the canvas"
    );
}

#[test]
fn the_inspector_names_no_store_type() {
    let source = include_str!("../src/editor/inspector.rs");
    for line in source.lines() {
        let t = line.trim();
        if t.starts_with("///") || t.starts_with("//") {
            continue;
        }
        assert!(
            !t.contains("decode_space"),
            "the inspector must not decode store records: {t}"
        );
        assert!(
            !t.contains("stored_at"),
            "the inspector must not read the store directly: {t}"
        );
        assert!(
            !t.contains(".records("),
            "the inspector must not query record ranges: {t}"
        );
    }
}

#[test]
fn selection_view_matches_the_inspector_fields() {
    let (_dir, store) = seeded();
    click(&store, addresses::NODE_A_KEY);
    let view = store.selection_view().expect("selection view");
    assert_eq!(view.address, "11100000");
    assert_eq!(view.style, "plain");
    assert_eq!(view.across, [0.4, 0.4, 0.0]);
    assert_eq!(view.down, [0.4, 0.4, 0.0]);
    assert_eq!(view.origin, [0.0, 0.0]);
    assert_eq!(view.depth, 3);
}
