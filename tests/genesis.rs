//! E4 — the discard test on the editor's own screen.
//!
//! Delete every space under the editor's screen root. Restart. The portal still
//! runs and the canvas is empty, with a finding that says so — not a crash, not
//! a black screen. Re-run genesis. The screen is bit-identical to before the
//! delete.

use infinite_solutions::editor;
use infinite_solutions::editor::addresses;
use infinite_solutions::facade;

/// The invariant D45's addressing rests on, checked rather than only written down.
///
/// `facade::significant_bits` recovers an address's depth from its last non-zero
/// nibble, which is only correct while no level's nibble is zero — children are
/// numbered from one. Break that (write a key like `0x11_01_00_00`, meaning "child 0
/// of child 1") and the recovered depth is wrong, `Addr::contains` starts answering
/// nonsense, and nothing else in the suite would say why. Every well-known key that
/// names a *thing* is listed here; the range bounds are not, because an exclusive end
/// is deliberately the next sibling's start rather than an address of its own.
#[test]
fn a_well_known_key_is_a_hierarchy() {
    let keys: &[(&str, &[u8], u32)] = &[
        ("SCREEN_ROOT", addresses::SCREEN_ROOT_KEY, 4),
        ("CANVAS", addresses::CANVAS_KEY, 8),
        ("INSPECTOR", addresses::INSPECTOR_KEY, 8),
        ("INSPECTOR_ADDR", addresses::INSPECTOR_ADDR_KEY, 12),
        ("NODE_A", addresses::NODE_A_KEY, 12),
        ("NODE_A1", addresses::NODE_A1_KEY, 16),
        ("NODE_A2", addresses::NODE_A2_KEY, 16),
        ("NODE_B", addresses::NODE_B_KEY, 12),
        ("WIRE_AB", addresses::WIRE_AB_KEY, 12),
        ("STYLE_PLAIN", addresses::STYLE_PLAIN_KEY, 8),
        ("STYLE_CANVAS", addresses::STYLE_CANVAS_KEY, 8),
        ("STYLE_WIRE", addresses::STYLE_WIRE_KEY, 8),
        ("BEHAVIOUR_ROOT", addresses::BEHAVIOUR_ROOT_KEY, 4),
        ("BEHAVIOUR_PROBE", addresses::BEHAVIOUR_PROBE_KEY, 8),
        ("BEHAVIOUR_DISPLACE", addresses::BEHAVIOUR_DISPLACE_KEY, 8),
        ("DRAG_FROM", addresses::DRAG_FROM_KEY, 8),
        ("CAMERA", addresses::CAMERA_KEY, 8),
        ("SELECT", addresses::SELECT_KEY, 8),
        ("GRAPH_ROOT", addresses::GRAPH_ROOT_KEY, 4),
    ];
    for (name, key, want) in keys {
        assert_eq!(key.len(), 4, "{name} is one store key wide");
        assert_eq!(
            facade::significant_bits(key),
            *want,
            "{name} = {key:02x?} is significant to {want} bits"
        );
    }

    // And the containments the screen's structure claims.
    let canvas = facade::presenter_addr(addresses::CANVAS_KEY);
    let a = facade::presenter_addr(addresses::NODE_A_KEY);
    let b = facade::presenter_addr(addresses::NODE_B_KEY);
    let wire = facade::presenter_addr(addresses::WIRE_AB_KEY);
    for (name, child) in [("node A", &a), ("node B", &b), ("the wire", &wire)] {
        assert!(canvas.contains(child), "the canvas contains {name}");
        assert!(!child.contains(&canvas), "and {name} does not contain it");
    }
    assert!(a.contains(&facade::presenter_addr(addresses::NODE_A1_KEY)));
    assert!(a.contains(&facade::presenter_addr(addresses::NODE_A2_KEY)));
    assert!(!b.contains(&facade::presenter_addr(addresses::NODE_A1_KEY)));
    assert!(
        !facade::presenter_addr(addresses::STYLE_PLAIN_KEY).contains(&a),
        "a style row is in another region entirely"
    );
}

#[test]
fn emptied_screen_is_a_finding_and_reseed_is_identical() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = facade::open(dir.path()).expect("open");
    editor::seed(|k| store.has(k), |k, v| store.put(k, v));

    let before = store.records(addresses::SCREEN_ROOT_KEY, addresses::SCREEN_END_KEY);
    assert!(!before.is_empty(), "genesis must write a screen");
    let styles_before = store.records(addresses::STYLE_ROOT_KEY, addresses::STYLE_END_KEY);
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

    store.delete_range(addresses::SCREEN_ROOT_KEY, addresses::SCREEN_END_KEY);
    drop(store);

    let store = facade::open(dir.path()).expect("reopen");
    let placed = store.place_now();
    assert!(
        placed.placed.is_empty(),
        "an emptied screen must place nothing"
    );
    let findings = store.last_findings();
    assert_eq!(findings.len(), 1, "emptied screen is one finding, not a crash");
    assert_eq!(findings[0].kind, "empty-screen");
    assert!(!findings[0].remedy.is_empty());
    assert_eq!(findings[0].site.as_bytes(), addresses::SCREEN_ROOT_KEY);

    editor::seed(|k| store.has(k), |k, v| store.put(k, v));
    let after = store.records(addresses::SCREEN_ROOT_KEY, addresses::SCREEN_END_KEY);
    assert_eq!(before, after, "re-run genesis is bit-identical");

    store.delete_range(addresses::STYLE_ROOT_KEY, addresses::STYLE_END_KEY);
    let placed = store.place_now();
    assert!(
        !placed.placed.is_empty(),
        "an emptied style table still places via the bootstrap default"
    );
    editor::seed(|k| store.has(k), |k, v| store.put(k, v));
    let styles_after = store.records(addresses::STYLE_ROOT_KEY, addresses::STYLE_END_KEY);
    assert_eq!(styles_before, styles_after, "re-run genesis restores styles bit-identically");
}
