//! E4 — the discard test on the editor's own screen.
//!
//! Delete every space under the editor's screen root. Restart. The portal still
//! runs and the canvas is empty, with a finding that says so — not a crash, not
//! a black screen. Re-run genesis. The screen is bit-identical to before the
//! delete.

use infinite_solutions::editor;
use infinite_solutions::editor::addresses;
use infinite_solutions::facade;

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
