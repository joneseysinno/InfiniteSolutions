//! E16.0 — Spec flatten green check.

use infinite_solutions::editor::addresses;
use infinite_solutions::editor::spec::{self, build};

#[test]
fn nested_spec_flattens_without_containment_field() {
    let tree = vec![build::area(
        "a",
        1,
        [0.4, 0.4, 0.0],
        [0.4, 0.4, 0.0],
        [0.0, 0.0],
        true,
    )
    .with_children(vec![build::area(
        "a1",
        1,
        [0.1, 0.1, 0.0],
        [0.1, 0.1, 0.0],
        [0.0, 0.0],
        false,
    )])];
    let flat = spec::flatten(addresses::canvas_key(), &tree);
    assert_eq!(flat.len(), 2);
    assert_eq!(flat[0].key, addresses::node_a_key());
    assert!(flat[1].key.starts_with(&flat[0].key));
    // Committed form is only address + payload — no parent vector on the record.
    assert!(!flat[0].payload.is_empty());
}
