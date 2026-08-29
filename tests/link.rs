//! E5 — the editor's behaviour composition links.

use infinite_solutions::editor;
use infinite_solutions::editor::addresses;
use infinite_solutions::facade;

#[test]
fn the_editor_behaviour_composition_links() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = facade::open(dir.path()).expect("open");
    editor::seed(|k| store.has(k), |k, v| store.put(k, v));

    let out = store.link_at(addresses::BEHAVIOUR_ROOT_KEY);
    assert!(
        !out.has_findings(),
        "the editor's behaviour must link: {:?}",
        out.findings
    );
    assert!(
        !out.value.steps.is_empty(),
        "link returns a plan, not a Result"
    );
    assert_eq!(out.value.steps.len(), 21, "twenty-one behaviour blocks, one step each");
}
