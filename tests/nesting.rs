//! O23 / D20 — zoom crosses the node/space seam.
//!
//! This is the check the platform's own thesis has never had. `CHARTER.md`:
//!
//! > A space contains nodes, and a node may itself be a space — the same entity can be
//! > both at once. Zoom is how you cross that seam.
//!
//! Nothing in the repository has ever exercised that, and finding 19 recorded why:
//! the facade canonicalized every address to four bytes, so `Addr::contains` was
//! satisfied only by equality and `place_group`'s descend guard compared a level
//! clamped to ~9 against a `prefix_bits()` that was always 32. A node inside a node
//! was therefore placed as a *sibling* of its parent, visible at every zoom, and no
//! amount of genesis depth changed that.
//!
//! **Verification of the check itself** (D41, `PRESENTER.md` §11's discipline): run
//! against the pre-D45 addressing, with the nested fixture seeded and nothing else
//! changed, `a_closed_space_does_not_show_its_interior` failed with
//!
//! ```text
//! placed: [[10, 00, 00, 01], [10, 00, 00, 10], [10, 00, 00, 11],
//!          [10, 00, 00, 12], [10, 00, 00, 20]]
//! ```
//!
//! — five flat siblings, node A's interior among them, at the resting camera; and
//! `the_address_of_an_interior_node_says_it_is_interior` failed on its first line,
//! *"the canvas contains node A"*. That is finding 19 as an assertion rather than a
//! paragraph, and it is the failure that was actually seen before the fix was written.

use infinite_solutions::editor;
use infinite_solutions::editor::addresses;
use infinite_solutions::facade;

const WIDTH: f64 = 800.0;
const HEIGHT: f64 = 600.0;

fn seeded() -> (tempfile::TempDir, facade::Store) {
    let dir = tempfile::TempDir::new().unwrap();
    let store = facade::open(dir.path()).expect("open store");
    editor::seed(|k| store.has(k), |k, v| store.put(k, v));
    editor::bind(&store);
    store.set_surface(0.0, 0.0, WIDTH, HEIGHT, 1.0);
    (dir, store)
}

fn placed_keys(store: &facade::Store) -> Vec<Vec<u8>> {
    store
        .place_now()
        .placed
        .iter()
        .map(|p| p.at.as_bytes().to_vec())
        .collect()
}

fn holds(keys: &[Vec<u8>], key: &[u8]) -> bool {
    keys.iter().any(|k| k.as_slice() == key)
}

#[test]
fn a_closed_space_does_not_show_its_interior() {
    // At the resting camera the canvas is open — you can see the two nodes on it —
    // and node A is not. Its interior exists in the store and is absent from the
    // screen, which is the whole of "collapsed, a space renders as a node".
    let (_dir, store) = seeded();
    let keys = placed_keys(&store);

    assert!(holds(&keys, addresses::CANVAS_KEY), "the canvas is placed");
    assert!(holds(&keys, addresses::NODE_A_KEY), "node A is placed");
    assert!(holds(&keys, addresses::NODE_B_KEY), "node B is placed");
    assert!(
        !holds(&keys, addresses::NODE_A1_KEY) && !holds(&keys, addresses::NODE_A2_KEY),
        "node A is closed at the resting camera, so its interior is not on screen; \
         placed: {keys:02x?}"
    );
}

#[test]
fn zoom_opens_a_space_and_reveals_the_nodes_inside_it() {
    // The claim itself. One camera change, no edit to any record, and a level of the
    // graph that was a single node becomes a populated space.
    let (_dir, store) = seeded();
    assert!(!holds(&placed_keys(&store), addresses::NODE_A1_KEY));

    store.zoom_to(addresses::NODE_A_KEY);
    let keys = placed_keys(&store);

    assert!(
        holds(&keys, addresses::NODE_A_KEY),
        "node A is still placed — entering a space does not delete the node; \
         placed: {keys:02x?}"
    );
    assert!(
        holds(&keys, addresses::NODE_A1_KEY) && holds(&keys, addresses::NODE_A2_KEY),
        "zoomed into node A, its two interior nodes are on screen; placed: {keys:02x?}"
    );
}

#[test]
fn an_interior_node_is_placed_inside_its_host_and_not_beside_it() {
    // The failure the old scheme produced was not "nothing appeared" — it was that
    // the interior nodes appeared as *siblings*, laid out beside their host. This is
    // the assertion that tells the two apart.
    let (_dir, store) = seeded();
    store.zoom_to(addresses::NODE_A_KEY);
    let placement = store.place_now();
    let find = |key: &[u8]| {
        placement
            .placed
            .iter()
            .find(|p| p.at.as_bytes() == key)
            .map(|p| p.rect)
    };
    let host = find(addresses::NODE_A_KEY).expect("node A is placed");
    let inner = find(addresses::NODE_A1_KEY).expect("node A's first interior node is placed");

    assert!(
        inner.min.x >= host.min.x - 1e-9
            && inner.min.y >= host.min.y - 1e-9
            && inner.max.x <= host.max.x + 1e-9
            && inner.max.y <= host.max.y + 1e-9,
        "the interior node lies within its host: inner {inner:?} host {host:?}"
    );
}

#[test]
fn the_address_of_an_interior_node_says_it_is_interior() {
    // Containment is a property of the address, not of a field someone remembered to
    // set. This is the half of D45 that is (a) rather than the descend rule, and it is
    // what a property inspector or a permission check (O10) would rely on.
    let a = facade::presenter_addr(addresses::NODE_A_KEY);
    let a1 = facade::presenter_addr(addresses::NODE_A1_KEY);
    let b = facade::presenter_addr(addresses::NODE_B_KEY);
    let canvas = facade::presenter_addr(addresses::CANVAS_KEY);

    assert!(canvas.contains(&a), "the canvas contains node A");
    assert!(a.contains(&a1), "node A contains its own interior node");
    assert!(!b.contains(&a1), "node B does not");
    assert!(!a1.contains(&a), "and containment is not symmetric");
    assert!(
        a.prefix_bits() > canvas.prefix_bits() && a1.prefix_bits() > a.prefix_bits(),
        "depth is readable from the address: canvas {} < A {} < A1 {}",
        canvas.prefix_bits(),
        a.prefix_bits(),
        a1.prefix_bits()
    );
}

#[test]
fn a_probe_inside_an_open_space_answers_with_the_interior_node() {
    // P1 in `hyper-ui` is the picture and the pointer disagreeing. Once a space opens,
    // the pointer has to follow it in.
    let (_dir, store) = seeded();
    store.zoom_to(addresses::NODE_A_KEY);
    let placement = store.place_now();
    let inner = placement
        .placed
        .iter()
        .find(|p| p.at.as_bytes() == addresses::NODE_A1_KEY)
        .expect("the interior node is placed")
        .rect;
    let mid_x = (inner.min.x + inner.max.x) * 0.5;
    let mid_y = (inner.min.y + inner.max.y) * 0.5;

    assert_eq!(
        store.probe_at(mid_x, mid_y).as_deref(),
        Some(addresses::NODE_A1_KEY),
        "a point inside the interior node answers with the interior node"
    );
}
