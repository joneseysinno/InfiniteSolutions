//! E13.5 — wiring by pointer commits a canvas wire and previews link findings.

use infinite_compositor::core::kind;
use infinite_solutions::editor;
use infinite_solutions::editor::addresses;
use infinite_solutions::editor::mint;
use infinite_solutions::facade::{self, decode_space, significant_bits};

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

fn wire_drag(store: &facade::Store, from: &[u8], to: &[u8], mismatch: bool) -> Vec<u8> {
    let minted = mint::next_child(store, &mint::parent_key(from)).expect("mint child");
    let (sx, sy) = center(store, from);
    let (tx, ty) = center(store, to);

    store.amend(addresses::WIRE_MODE_KEY, &[1]);
    if mismatch {
        store.amend(addresses::WIRE_MISMATCH_KEY, &[1]);
    }
    store.amend(addresses::POINTER_POSITION.as_bytes(), &point(sx, sy));
    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[1]);
    editor::run(store);

    if mismatch {
        store.amend(addresses::WIRE_MISMATCH_KEY, &[1]);
    }
    store.amend(addresses::POINTER_POSITION.as_bytes(), &point(tx, ty));
    editor::run(store);

    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[0]);
    store.amend(addresses::RELEASE_PULSE_KEY, &[1]);
    editor::run(store);

    store.commit_at(&minted);
    store.commit_at(addresses::GRAPH_ROOT_KEY);
    drain(store);
    minted
}

#[test]
fn dragging_between_two_nodes_commits_a_wire_under_their_parent() {
    let (_dir, store) = seeded();
    let minted = wire_drag(
        &store,
        addresses::NODE_A_KEY,
        addresses::NODE_B_KEY,
        false,
    );

    assert_eq!(
        minted,
        vec![0x11, 0x40, 0x00, 0x00],
        "fourth canvas sibling after A, B, and genesis wire"
    );
    assert_eq!(
        mint::parent_key(&minted),
        addresses::CANVAS_KEY.to_vec(),
        "the wire lives on the canvas beside its endpoints"
    );
    assert_eq!(
        significant_bits(&minted),
        significant_bits(addresses::NODE_A_KEY),
        "the wire is a sibling of the endpoints on the canvas"
    );

    let space = decode_space(&store.stored_at(&minted).expect("wire stored")).expect("IS1");
    assert_eq!(space.primitive, "wire");
    assert_eq!(
        space.link,
        Some((
            addresses::NODE_A_KEY.to_vec(),
            addresses::NODE_B_KEY.to_vec()
        ))
    );
}

#[test]
fn an_authored_wire_survives_restart() {
    let (dir, store) = seeded();
    let minted = wire_drag(
        &store,
        addresses::NODE_A_KEY,
        addresses::NODE_B_KEY,
        false,
    );
    assert!(store.has(&minted));
    drop(store);

    let store = facade::open(dir.path()).expect("reopen");
    editor::bind(&store);
    assert!(store.has(&minted), "genesis put_if must not overwrite user wires");
}

#[test]
fn a_mismatch_finding_appears_before_release_and_zooms_to_its_site() {
    let (_dir, store) = seeded();
    let (sx, sy) = center(&store, addresses::NODE_A_KEY);
    let (tx, ty) = center(&store, addresses::NODE_B_KEY);

    store.amend(addresses::WIRE_MODE_KEY, &[1]);
    store.amend(addresses::WIRE_MISMATCH_KEY, &[1]);
    store.amend(addresses::POINTER_POSITION.as_bytes(), &point(sx, sy));
    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[1]);
    editor::run(&store);

    store.amend(addresses::POINTER_POSITION.as_bytes(), &point(tx, ty));
    store.amend(addresses::WIRE_MISMATCH_KEY, &[1]);
    editor::run(&store);

    assert!(
        store.stored_at(addresses::GRAPH_ROOT_KEY).is_none(),
        "C4: the preview graph is pending, not committed"
    );
    let _ = store.place_now();
    let findings = store.last_findings();
    let mismatch: Vec<_> = findings
        .iter()
        .filter(|f| f.kind == kind::TAG_MISMATCH)
        .collect();
    assert_eq!(
        mismatch.len(),
        1,
        "exactly one tag-mismatch before release: {:?}",
        findings
    );
    assert_eq!(mismatch[0].site.as_bytes(), addresses::NODE_B_KEY);

    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[0]);
    store.amend(addresses::RELEASE_PULSE_KEY, &[1]);
    editor::run(&store);

    let minted = mint::next_child(&store, addresses::CANVAS_KEY).expect("minted wire");
    store.commit_at(&minted);
    store.commit_at(addresses::GRAPH_ROOT_KEY);
    drain(&store);

    assert!(
        store.stored_at(addresses::GRAPH_ROOT_KEY).is_some(),
        "D21: releasing still commits the preview graph"
    );

    let before = store.camera();
    store.zoom_to(mismatch[0].site.as_bytes());
    assert!(
        store.camera().zoom > before.zoom,
        "clicking the finding zooms to its site"
    );
}
