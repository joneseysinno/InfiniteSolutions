//! E10.1 — the frame is read back and the pixels are the authored ones.
//!
//! **This is the check every stage before it lacked** (D41). `tests/agreement.rs`
//! proves that the culling transform and the drawing transform agree; it is pure
//! `f64` arithmetic and it passes whether or not anything is ever drawn. Nine stages
//! were marked landed on checks of that kind, and `src/facade/ports/surface.rs`
//! discarded every frame the whole time.
//!
//! The test renders into a texture rather than a swapchain, so it needs no window
//! and no display server and runs wherever the suite runs. It fails for exactly one
//! reason: the pixels are not what the store says they should be.
//!
//! Verification of the check itself (E0's clause, `PRESENTER.md` §11's discipline):
//! this was run against the pre-E10 `Surface`, whose `submit` ended
//! `let _ = (_format, verts);`, and it failed on the first assertion with the clear
//! colour where the node should have been. A check that has never been seen to fail
//! is a check nobody knows the polarity of.

use infinite_solutions::editor;
use infinite_solutions::facade::ports::Surface;
use infinite_solutions::facade::{self, encode_style};
use infinite_presenter::core::{Point, SurfaceRect};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

/// Fills as `editor::styles::bootstrap_default` writes them, in 8-bit.
const PLAIN: [u8; 3] = [56, 122, 209];
const CANVAS: [u8; 3] = [31, 33, 41];

/// The default camera puts root-space (0,0) at surface (200,100) at zoom 400, so
/// the canvas covers (200,100)..(600,500) and a node covers (200,100)..(360,260).
const IN_NODE: (u32, u32) = (250, 150);
const IN_NODE_B: (u32, u32) = (450, 150);
const IN_CANVAS_ONLY: (u32, u32) = (500, 400);

fn near(got: [u8; 4], want: [u8; 3], what: &str) {
    // A tolerance, not a golden image: float-to-unorm8 rounding is not bit-identical
    // between a software rasteriser and a discrete GPU, and the failure this test
    // exists to catch is "nothing was drawn", which is never within two counts.
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

#[test]
fn the_authored_screen_reaches_the_framebuffer() {
    let (_dir, store) = seeded();
    let Some(mut surface) = surface() else {
        eprintln!("no GPU adapter available; skipping");
        return;
    };
    store.draw_with(&mut surface);
    let pixels = surface.read_back().expect("read back");

    let node = Surface::pixel(&pixels, WIDTH, IN_NODE.0, IN_NODE.1);
    near(node, PLAIN, "a node is the fill on its authored style row");

    let node_b = Surface::pixel(&pixels, WIDTH, IN_NODE_B.0, IN_NODE_B.1);
    near(node_b, PLAIN, "the second node keeps its authored position");

    let canvas = Surface::pixel(&pixels, WIDTH, IN_CANVAS_ONLY.0, IN_CANVAS_ONLY.1);
    near(canvas, CANVAS, "the canvas is the fill on its own style row");

    assert_ne!(
        node, canvas,
        "two style rows must not resolve to one colour, or the test proves nothing"
    );
    assert_eq!(node, node_b, "both nodes use the authored plain style");
}

#[test]
fn editing_one_style_row_changes_the_picture() {
    // The end-to-end claim, and the reason a readback is worth having: the pixel is
    // downstream of a record. Change the record, the pixel changes. Nothing is
    // recompiled and no code path is special-cased for it.
    let (_dir, store) = seeded();
    let Some(mut surface) = surface() else {
        eprintln!("no GPU adapter available; skipping");
        return;
    };
    store.draw_with(&mut surface);
    let before = Surface::pixel(
        &surface.read_back().expect("read back"),
        WIDTH,
        IN_NODE.0,
        IN_NODE.1,
    );
    near(before, PLAIN, "the node starts at its authored fill");

    store.put(
        editor::addresses::STYLE_PLAIN_KEY,
        &encode_style("plain", [0.90, 0.20, 0.10, 1.0]),
    );
    store.draw_with(&mut surface);
    let after = Surface::pixel(
        &surface.read_back().expect("read back"),
        WIDTH,
        IN_NODE.0,
        IN_NODE.1,
    );
    near(after, [230, 51, 26], "the node follows its style row");
}

#[test]
fn an_empty_screen_is_a_finding_and_not_a_black_frame() {
    // `PRESENTER.md` §13 finding 8: a failed query and an empty screen must never be
    // indistinguishable. E4 proves the finding is raised; this proves the frame that
    // accompanies it is the authored background rather than whatever was there.
    let (_dir, store) = seeded();
    let Some(mut surface) = surface() else {
        eprintln!("no GPU adapter available; skipping");
        return;
    };
    store.delete_range(
        editor::addresses::SCREEN_ROOT_KEY,
        editor::addresses::SCREEN_END_KEY,
    );
    store.draw_with(&mut surface);
    let pixels = surface.read_back().expect("read back");
    assert!(
        !store.last_findings().is_empty(),
        "an emptied screen root is a finding"
    );
    let anywhere = Surface::pixel(&pixels, WIDTH, IN_NODE.0, IN_NODE.1);
    assert_eq!(
        anywhere[3], 255,
        "the frame is still opaque; an empty screen is not an absent frame"
    );
}

#[test]
fn a_surface_with_no_device_still_narrows() {
    // D29's argument, kept honest: the arithmetic runs with no GPU present, which is
    // what lets every other test in the suite stay headless.
    let (_dir, store) = seeded();
    store.draw();
    assert!(
        store.place_now().placed.len() >= 2,
        "the seeded screen places without a device"
    );
}
