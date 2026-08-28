//! E11 — a wire is a line on the screen, not only a row in the store.
//!
//! Before this, a wire existed as data all the way through: linked, validated,
//! mismatch-findable, zoom-to-site-able — and nothing drew one. `EDITOR-BOOTSTRAP.md`
//! §1's six interactions listed *wire* among them and the six were never all true at
//! once.
//!
//! The check is stated the E10 way (D41): draw a known composition with one wire,
//! read the framebuffer back, and assert the wire's colour is on the pixel path
//! between its two endpoints — not that the draw call failed to panic.
//!
//! **Verification of the check itself** (D41): both halves were run and both were
//! seen to fail, for their own reason.
//!
//! - With `genesis`'s wire record not written,
//!   `a_wire_is_drawn_between_the_two_nodes_it_joins` failed at *"the wire is placed
//!   with two endpoints"* — nothing to draw, and the test says so rather than passing
//!   vacuously.
//! - With D46's batching defeated, so the link batch fell through to the quad
//!   pipeline, `off_the_line_is_still_the_canvas` failed with
//!   `channel 0 was 242, wanted 31 (whole pixel [242, 181, 51, 255])` — the wire's
//!   colour forty pixels off the line, which is a bounding box and not a line.

use infinite_solutions::editor;
use infinite_solutions::editor::addresses;
use infinite_solutions::facade::ports::Surface;
use infinite_solutions::facade;
use infinite_presenter::core::{Point, SurfaceRect};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

/// Fills as `editor::styles::bootstrap_default` writes them, in 8-bit.
const WIRE: [u8; 3] = [242, 181, 51];
const CANVAS: [u8; 3] = [31, 33, 41];

fn near(got: [u8; 4], want: [u8; 3], what: &str) {
    for channel in 0..3 {
        let d = i32::from(got[channel]) - i32::from(want[channel]);
        assert!(
            d.abs() <= 2,
            "{what}: channel {channel} was {}, wanted {} (whole pixel {got:?})",
            got[channel],
            want[channel]
        );
    }
}

fn seeded() -> (tempfile::TempDir, facade::Store) {
    let dir = tempfile::TempDir::new().unwrap();
    let store = facade::open(dir.path()).expect("open store");
    editor::seed(|k| store.has(k), |k, v| store.put(k, v));
    editor::bind(&store);
    store.set_surface(0.0, 0.0, f64::from(WIDTH), f64::from(HEIGHT), 1.0);
    (dir, store)
}

fn surface() -> Option<Surface> {
    Surface::offscreen(
        WIDTH,
        HEIGHT,
        SurfaceRect::new(
            Point::ORIGIN,
            Point::new(f64::from(WIDTH), f64::from(HEIGHT)),
            1.0,
        ),
    )
}

/// The two surface points the wire runs between, taken from the placement rather
/// than hardcoded — so the test follows the camera instead of asserting a constant
/// that a change to genesis would silently invalidate.
fn ends(store: &facade::Store) -> ((f64, f64), (f64, f64)) {
    let placement = store.place_now();
    let span = placement
        .placed
        .iter()
        .find(|p| p.at.as_bytes() == addresses::WIRE_AB_KEY)
        .and_then(|p| p.span)
        .expect("the wire is placed with two endpoints");
    ((span.0.x, span.0.y), (span.1.x, span.1.y))
}

#[test]
fn the_placement_groups_the_wire_apart_from_the_rectangles() {
    // D46. The presenter authors the grouping; without it the facade would have to
    // work out for itself which entries are lines, which is finding 16's whole point.
    let (_dir, store) = seeded();
    let placement = store.place_now();

    let total: usize = placement.batches.iter().map(|b| b.count).sum();
    assert_eq!(
        total,
        placement.placed.len(),
        "the batches are a partition of what was placed"
    );
    let link = placement
        .batches
        .iter()
        .find(|b| &*b.primitive == "wire")
        .expect("the wire is in a batch of its own");
    assert!(
        placement.placed[link.first..link.first + link.count]
            .iter()
            .all(|p| p.span.is_some()),
        "every entry in a link batch carries the two points it runs between"
    );
    assert!(
        placement
            .batches
            .iter()
            .filter(|b| &*b.primitive == "rect")
            .flat_map(|b| &placement.placed[b.first..b.first + b.count])
            .all(|p| p.span.is_none()),
        "and no area does"
    );
}

#[test]
fn a_wire_is_drawn_between_the_two_nodes_it_joins() {
    let (_dir, store) = seeded();
    let Some(mut surface) = surface() else {
        eprintln!("no GPU adapter available; skipping");
        return;
    };
    let (a, b) = ends(&store);
    store.draw_with(&mut surface);
    let pixels = surface.read_back().expect("read back");

    // Five samples along the segment, endpoints excluded — the endpoints sit inside
    // the nodes, where a wire-coloured pixel would prove nothing about the line.
    for step in 1..=5 {
        let t = f64::from(step) / 6.0;
        let x = (a.0 + (b.0 - a.0) * t).round() as u32;
        let y = (a.1 + (b.1 - a.1) * t).round() as u32;
        near(
            Surface::pixel(&pixels, WIDTH, x, y),
            WIRE,
            &format!("the wire's colour is on the path between its ends, at ({x}, {y})"),
        );
    }
}

#[test]
fn off_the_line_is_still_the_canvas() {
    // The other half of the claim, and the one that fails if a wire is drawn as its
    // bounding box: a line has to be thin.
    let (_dir, store) = seeded();
    let Some(mut surface) = surface() else {
        eprintln!("no GPU adapter available; skipping");
        return;
    };
    let (a, b) = ends(&store);
    store.draw_with(&mut surface);
    let pixels = surface.read_back().expect("read back");

    let mid_x = ((a.0 + b.0) * 0.5).round() as u32;
    let above = ((a.1 + b.1) * 0.5 - 40.0).round() as u32;
    let below = ((a.1 + b.1) * 0.5 + 40.0).round() as u32;
    near(
        Surface::pixel(&pixels, WIDTH, mid_x, above),
        CANVAS,
        "forty pixels above the wire is canvas",
    );
    near(
        Surface::pixel(&pixels, WIDTH, mid_x, below),
        CANVAS,
        "forty pixels below the wire is canvas",
    );

    // The corner of the wire's own bounding box, well clear of the diagonal. This is
    // the sample that tells a line from the box `Placed::rect` reports for it, and it
    // is the reason genesis authors node B down as well as across: with the two nodes
    // in a row, the box *is* the line and no sample could tell them apart.
    let corner_x = (a.0.min(b.0) + 24.0).round() as u32;
    let corner_y = (a.1.max(b.1) - 24.0).round() as u32;
    near(
        Surface::pixel(&pixels, WIDTH, corner_x, corner_y),
        CANVAS,
        "the far corner of the wire's bounding box is canvas, so a line was drawn \
         and not a rectangle",
    );
}

#[test]
fn deleting_the_wire_record_takes_the_line_off_the_screen() {
    // The end-to-end claim, the way `editing_one_style_row_changes_the_picture` makes
    // it for a colour: the line is downstream of a record and of nothing else.
    let (_dir, store) = seeded();
    let Some(mut surface) = surface() else {
        eprintln!("no GPU adapter available; skipping");
        return;
    };
    let (a, b) = ends(&store);
    let mid_x = ((a.0 + b.0) * 0.5).round() as u32;
    let mid_y = ((a.1 + b.1) * 0.5).round() as u32;

    store.draw_with(&mut surface);
    near(
        Surface::pixel(&surface.read_back().expect("read back"), WIDTH, mid_x, mid_y),
        WIRE,
        "the wire starts on screen",
    );

    store.delete_key(addresses::WIRE_AB_KEY);
    store.draw_with(&mut surface);
    near(
        Surface::pixel(&surface.read_back().expect("read back"), WIDTH, mid_x, mid_y),
        CANVAS,
        "and goes when its record does",
    );
}

#[test]
fn a_wire_with_an_endpoint_off_screen_is_not_placed() {
    // A hyperedge has no geometry of its own: it borrows its ends'. When an end is
    // not on screen there is no honest line to draw, and drawing one anyway is how a
    // wire ends up pointing at nothing.
    let (_dir, store) = seeded();
    store.delete_key(addresses::NODE_B_KEY);
    let placement = store.place_now();
    assert!(
        placement
            .placed
            .iter()
            .all(|p| p.at.as_bytes() != addresses::WIRE_AB_KEY),
        "with one end gone the wire is not placed"
    );
    let total: usize = placement.batches.iter().map(|b| b.count).sum();
    assert_eq!(
        total,
        placement.placed.len(),
        "and the batches still partition what was placed"
    );
}
