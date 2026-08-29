//! E4 — the discard test on the editor's own screen.
//!
//! Delete every space under the editor's screen root. Restart. The portal still
//! runs and the canvas is empty, with a finding that says so — not a crash, not
//! a black screen. Re-run genesis. The screen is bit-identical to before the
//! delete.

use infinite_solutions::editor;
use infinite_solutions::editor::addresses;
use infinite_solutions::editor::mint;
use infinite_solutions::facade;

/// E15 hierarchy: significant length is `8 × key.len()`, and children are parent
/// with one more 2-byte slot.
#[test]
fn a_well_known_key_is_a_hierarchy() {
    let keys: &[(&str, &[u8], u32)] = &[
        ("SCREEN_ROOT", addresses::SCREEN_ROOT_KEY, 8),
        ("CANVAS", addresses::canvas_key(), 24),
        ("INSPECTOR", addresses::inspector_key(), 24),
        ("INSPECTOR_ADDR", addresses::inspector_addr_key(), 40),
        ("TOOLBAR", addresses::toolbar_key(), 24),
        ("TOOLBAR_HISTORY", addresses::toolbar_history_key(), 40),
        ("NODE_A", addresses::node_a_key(), 40),
        ("NODE_A1", addresses::node_a1_key(), 56),
        ("NODE_A2", addresses::node_a2_key(), 56),
        ("NODE_B", addresses::node_b_key(), 40),
        ("WIRE_AB", addresses::wire_ab_key(), 40),
        ("STYLE_PLAIN", addresses::style_plain_key(), 24),
        ("STYLE_CANVAS", addresses::style_canvas_key(), 24),
        ("STYLE_WIRE", addresses::style_wire_key(), 24),
        ("BEHAVIOUR_ROOT", addresses::BEHAVIOUR_ROOT_KEY, 8),
        ("BEHAVIOUR_PROBE", addresses::behaviour_probe_key(), 24),
        ("BEHAVIOUR_DISPLACE", addresses::behaviour_displace_key(), 24),
        ("DRAG_FROM", addresses::drag_from_key(), 24),
        ("CAMERA", addresses::camera_key(), 24),
        ("SELECT", addresses::select_key(), 24),
        ("RUN", addresses::run_key(), 24),
        ("GRAPH_ROOT", addresses::GRAPH_ROOT_KEY, 8),
    ];
    for (name, key, want) in keys {
        assert_eq!(
            facade::bits_of(key),
            *want,
            "{name} = {key:02x?} is significant to {want} bits"
        );
        if *want > 8 {
            let (derived, bits) = mint::child(
                &key[..key.len() - 2],
                facade::bits_of(&key[..key.len() - 2]),
                u32::from(key[key.len() - 2]) << 8 | u32::from(key[key.len() - 1]),
            )
            .expect("re-derive");
            assert_eq!(derived.as_slice(), *key, "{name} re-derives");
            assert_eq!(bits, *want);
        }
    }

    let canvas = facade::presenter_addr(addresses::canvas_key());
    let a = facade::presenter_addr(addresses::node_a_key());
    let b = facade::presenter_addr(addresses::node_b_key());
    let wire = facade::presenter_addr(addresses::wire_ab_key());
    for (name, child) in [("node A", &a), ("node B", &b), ("the wire", &wire)] {
        assert!(canvas.contains(child), "the canvas contains {name}");
        assert!(!child.contains(&canvas), "and {name} does not contain it");
    }
    assert!(a.contains(&facade::presenter_addr(addresses::node_a1_key())));
    assert!(a.contains(&facade::presenter_addr(addresses::node_a2_key())));
    assert!(!b.contains(&facade::presenter_addr(addresses::node_a1_key())));
    assert!(
        !facade::presenter_addr(addresses::style_plain_key()).contains(&a),
        "a style row is in another region entirely"
    );
}

#[test]
fn emptied_screen_is_a_finding_and_reseed_is_identical() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = facade::open(dir.path()).expect("open");
    editor::seed(|k| store.has(k), |k, v| store.put(k, v));

    let before = store.records(addresses::SCREEN_ROOT_KEY, addresses::screen_end_key());
    assert!(!before.is_empty(), "genesis must write a screen");
    let styles_before = store.records(addresses::STYLE_ROOT_KEY, addresses::style_end_key());
    assert!(!styles_before.is_empty(), "D37: genesis authors the style table");
    let placed = store.place_now();
    assert!(!placed.placed.is_empty(), "a seeded screen must place");
    assert!(
        store
            .last_findings()
            .iter()
            .all(|f| f.kind != "empty-screen"),
        "a seeded screen is not an empty-screen finding"
    );

    store.delete_range(addresses::SCREEN_ROOT_KEY, addresses::screen_end_key());
    drop(store);

    let store = facade::open(dir.path()).expect("reopen");
    let placed = store.place_now();
    assert!(
        placed.placed.is_empty(),
        "emptied screen places nothing"
    );
    assert!(
        store
            .last_findings()
            .iter()
            .any(|f| f.kind == "empty-screen"),
        "emptied screen raises empty-screen"
    );

    editor::seed(|k| store.has(k), |k, v| store.put(k, v));
    let after = store.records(addresses::SCREEN_ROOT_KEY, addresses::screen_end_key());
    assert_eq!(before, after, "reseed is bit-identical");
}

#[test]
fn genesis_rs_is_under_one_hundred_fifty_lines() {
    let src = include_str!("../src/editor/genesis.rs");
    let lines = src.lines().count();
    assert!(
        lines < 150,
        "genesis.rs is {lines} lines; E16.1 requires under 150"
    );
}
