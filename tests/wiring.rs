//! E8 — a wire is linked while still pending (C4), and the finding surface zooms.
//!
//! Drag a wire between two spaces with mismatched tags. The finding appears
//! before release. Releasing still creates the edge. Clicking the finding
//! zooms to its site.

use infinite_compositor::core::kind;
use infinite_solutions::editor;
use infinite_solutions::editor::addresses;
use infinite_solutions::editor::tags;
use infinite_solutions::facade::{
    self, decode_composition, encode_composition, BlockRecord, CompositionRecord, PortRecord,
    WireRecord,
};

fn mismatch_graph() -> Vec<u8> {
    encode_composition(&CompositionRecord {
        compilable: false,
        blocks: vec![
            BlockRecord {
                at: addresses::NODE_A_KEY.to_vec(),
                kind: "native".into(),
                target: b"offset".to_vec(),
                ports: vec![
                    port("from", true, tags::POINT, false),
                    port("to", true, tags::POINT, false),
                    port("delta", false, tags::POINT, false),
                ],
            },
            BlockRecord {
                at: addresses::NODE_B_KEY.to_vec(),
                kind: "native".into(),
                target: b"commit".to_vec(),
                ports: vec![
                    port("addr", true, tags::ADDRESS, true),
                    port("done", false, tags::FLAG, false),
                ],
            },
        ],
        wires: vec![WireRecord {
            sources: vec![(addresses::NODE_A_KEY.to_vec(), "delta".into())],
            sinks: vec![(addresses::NODE_B_KEY.to_vec(), "addr".into())],
        }],
    })
}

fn port(name: &str, incoming: bool, tag: &str, required: bool) -> PortRecord {
    PortRecord {
        name: name.into(),
        incoming,
        tag: tag.into(),
        arity: None,
        required,
    }
}

fn persist_graph(store: &facade::Store) {
    store.commit_at(addresses::GRAPH_ROOT_KEY);
    for _ in 0..32 {
        if store.committed_len() == 0 {
            break;
        }
        store.tick();
    }
    store.sync().expect("sync");
}

#[test]
fn a_pending_wire_links_before_commit_and_a_mismatch_zooms_to_its_site() {
    let dir = tempfile::TempDir::new().unwrap();
    let store = facade::open(dir.path()).expect("open");
    editor::seed(|k| store.has(k), |k, v| store.put(k, v));
    editor::bind(&store);
    store.set_surface(0.0, 0.0, 800.0, 600.0, 1.0);
    let _ = store.place_now();

    let payload = mismatch_graph();
    assert!(
        store.amend(addresses::GRAPH_ROOT_KEY, &payload),
        "the in-flight wire is pending, never a write"
    );
    assert!(
        store.stored_at(addresses::GRAPH_ROOT_KEY).is_none(),
        "C4: the wire is not committed"
    );

    let preview = store.link_at(addresses::GRAPH_ROOT_KEY);
    let mismatch: Vec<_> = preview
        .findings
        .iter()
        .filter(|f| f.kind == kind::TAG_MISMATCH)
        .collect();
    assert_eq!(
        mismatch.len(),
        1,
        "exactly one tag-mismatch, not a stack trace: {:?}",
        preview.findings
    );
    let finding = mismatch[0];
    assert_eq!(finding.site.as_bytes(), addresses::NODE_B_KEY);
    assert!(!finding.said.is_empty());
    assert!(!finding.wanted.is_empty());
    assert!(!finding.remedy.is_empty());
    assert_ne!(finding.said, finding.wanted);

    persist_graph(&store);
    let stored = store
        .stored_at(addresses::GRAPH_ROOT_KEY)
        .expect("D21: releasing still creates the edge");
    let composition = decode_composition(&stored).expect("CM1");
    assert_eq!(composition.wires.len(), 1, "the mismatched wire was kept");

    let before = store.camera();
    store.zoom_to(finding.site.as_bytes());
    let after = store.camera();
    assert!(
        after.zoom > before.zoom,
        "clicking the finding zooms to the site"
    );
}
