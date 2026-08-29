//! E18b.1 — two screens delegate to one stored definition.

use infinite_solutions::editor;
use infinite_solutions::editor::addresses;
use infinite_solutions::editor::tags;
use infinite_solutions::facade::{
    self, encode_composition, BlockRecord, CompositionRecord, PortRecord,
};

fn seeded() -> (tempfile::TempDir, facade::Store) {
    let dir = tempfile::TempDir::new().unwrap();
    let store = facade::open(dir.path()).expect("open");
    editor::seed(|k| store.has(k), |k, v| store.put(k, v));
    editor::bind(&store);
    (dir, store)
}

fn screen_key(slot: u32) -> Vec<u8> {
    addresses::child_key(addresses::GRAPH_ROOT_KEY, slot)
}

fn delegate_screen(at: &[u8], def: &[u8]) -> Vec<u8> {
    encode_composition(&CompositionRecord {
        compilable: false,
        blocks: vec![BlockRecord {
            at: at.to_vec(),
            kind: "delegate".into(),
            target: def.to_vec(),
            ports: Vec::new(),
        }],
        wires: Vec::new(),
    })
}

fn extra_map(def: &[u8]) -> Vec<u8> {
    encode_composition(&CompositionRecord {
        compilable: true,
        blocks: vec![
            native(
                addresses::field_row_map_key(),
                b"map",
                vec![
                    port("fn", true, tags::VALUE, false),
                    port("val", true, tags::VALUE, false),
                    port("aux", true, tags::VALUE, false),
                    port("out", false, tags::VALUE, false),
                ],
            ),
            native(
                addresses::field_row_fold_key(),
                b"fold",
                vec![
                    port("fn", true, tags::VALUE, false),
                    port("left", true, tags::VALUE, false),
                    port("right", true, tags::VALUE, false),
                    port("out", false, tags::VALUE, false),
                ],
            ),
            native(
                &addresses::child_key(def, 3),
                b"map",
                vec![
                    port("fn", true, tags::VALUE, false),
                    port("val", true, tags::VALUE, false),
                    port("aux", true, tags::VALUE, false),
                    port("out", false, tags::VALUE, false),
                ],
            ),
        ],
        wires: Vec::new(),
    })
}

fn native(at: &[u8], key: &[u8], ports: Vec<PortRecord>) -> BlockRecord {
    BlockRecord {
        at: at.to_vec(),
        kind: "native".into(),
        target: key.to_vec(),
        ports,
    }
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

#[test]
fn two_screens_share_one_store_definition() {
    let (_dir, store) = seeded();
    assert!(
        store.has(addresses::field_row_def_key()),
        "the field_row definition is store data"
    );
    assert!(
        store.has(addresses::app_root_key()),
        "src/ seeds a delegate block (kind != native)"
    );

    let a = screen_key(7);
    let b = screen_key(8);
    store.put(
        &a,
        &delegate_screen(&a, addresses::field_row_def_key()),
    );
    store.put(
        &b,
        &delegate_screen(&b, addresses::field_row_def_key()),
    );

    let before_a = store.link_at(&a);
    let before_b = store.link_at(&b);
    assert!(
        !before_a.has_findings() && !before_b.has_findings(),
        "both screens link: {:?} / {:?}",
        before_a.findings,
        before_b.findings
    );
    assert_eq!(
        before_a.value.steps.len(),
        before_b.value.steps.len(),
        "both screens inline the same definition"
    );
    let start = before_a.value.steps.len();
    assert!(start >= 2, "the shared definition is not empty");

    store.put(
        addresses::field_row_def_key(),
        &extra_map(addresses::field_row_def_key()),
    );
    let after_a = store.link_at(&a);
    let after_b = store.link_at(&b);
    assert_eq!(after_a.value.steps.len(), start + 1);
    assert_eq!(
        after_b.value.steps.len(),
        after_a.value.steps.len(),
        "editing the store definition changes both screens; no recompile"
    );
}
