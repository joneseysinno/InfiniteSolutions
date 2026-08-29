//! E13.0 — a text primitive reaches the screen.
//!
//! The third key under D46: `primitive: "text"`, measured through the `Glyphs` port
//! and drawn as ink cells. A readback asserts known string pixels differ from the
//! background in the cells layout names, and match at a second scale factor.

use infinite_presenter::core::{Point, SurfaceRect, TEXT};
use infinite_solutions::editor;
use infinite_solutions::facade::ports::{Glyphs, Surface};
use infinite_solutions::facade::{self, encode_space, SpaceRecord};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

/// A label on the canvas, sibling to the genesis nodes.
const LABEL_KEY: &[u8] = &[0x11, 0x40, 0x00, 0x00];

/// Fills as `editor::styles::bootstrap_default` writes them, in 8-bit.
const PLAIN: [u8; 3] = [56, 122, 209];
const CANVAS: [u8; 3] = [31, 33, 41];

const RUN: &str = "Hi";

/// Canvas-only sample from `tests/pixels.rs` — avoids genesis nodes and the wire.
const IN_CANVAS_ONLY: (u32, u32) = (250, 450);

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

fn label_record() -> SpaceRecord {
    SpaceRecord {
        across: [0.0, 0.0, 0.0],
        down: [0.0, 0.05, 0.0],
        style: "plain".into(),
        detail_override: None,
        hosts_space: false,
        accepts: true,
        origin: [0.05, 0.05],
        primitive: TEXT.into(),
        link: None,
        text: RUN.into(),
    }
}

fn seeded() -> (tempfile::TempDir, facade::Store) {
    let dir = tempfile::TempDir::new().unwrap();
    let store = facade::open(dir.path()).expect("open store");
    editor::seed(|k| store.has(k), |k, v| store.put(k, v));
    store.put(LABEL_KEY, &encode_space(&label_record()));
    editor::bind(&store);
    store.set_surface(0.0, 0.0, f64::from(WIDTH), f64::from(HEIGHT), 1.0);
    (dir, store)
}

fn surface(scale: f64) -> Option<Surface> {
    Surface::offscreen(
        WIDTH,
        HEIGHT,
        SurfaceRect::new(
            Point::ORIGIN,
            Point::new(f64::from(WIDTH), f64::from(HEIGHT)),
            scale,
        ),
    )
}

fn ink_sample(store: &facade::Store, scale: f64) -> (u32, u32, [u8; 4]) {
    let placement = store.place_now();
    let placed = placement
        .placed
        .iter()
        .find(|p| p.at.as_bytes() == LABEL_KEY)
        .expect("the label is placed");
    let showing = placed.showing();
    let em = (showing.max.y - showing.min.y).max(1e-12);
    let cell = Glyphs::ink_cells(RUN, showing.min, em)
        .next()
        .expect("the run has at least one lit cell");
    let logical = Point::new(
        (cell.min.x + cell.max.x) * 0.5,
        (cell.min.y + cell.max.y) * 0.5,
    );
    let x = (logical.x * scale).round().clamp(0.0, f64::from(WIDTH - 1)) as u32;
    let y = (logical.y * scale).round().clamp(0.0, f64::from(HEIGHT - 1)) as u32;
    let Some(mut surf) = surface(scale) else {
        panic!("no GPU adapter available");
    };
    store.set_surface(0.0, 0.0, f64::from(WIDTH), f64::from(HEIGHT), scale);
    store.draw_with(&mut surf);
    let pixels = surf.read_back().expect("read back");
    (x, y, Surface::pixel(&pixels, WIDTH, x, y))
}

#[test]
fn a_text_run_reaches_the_framebuffer_at_two_scales() {
    let (_dir, store) = seeded();
    let Some(mut surf) = surface(1.0) else {
        eprintln!("no GPU adapter available; skipping");
        return;
    };
    store.draw_with(&mut surf);
    let pixels = surf.read_back().expect("read back");

    let (_, _, ink) = ink_sample(&store, 1.0);
    near(ink, PLAIN, "a lit cell is the fill on its style row");

    let background = Surface::pixel(&pixels, WIDTH, IN_CANVAS_ONLY.0, IN_CANVAS_ONLY.1);
    near(background, CANVAS, "off the run is still the canvas");
    assert_ne!(
        ink, background,
        "text ink and background must differ, or the test proves nothing"
    );

    let (_, _, ink2) = ink_sample(&store, 2.0);
    near(ink2, PLAIN, "the same cell at scale 2.0 keeps the authored fill");
    assert_eq!(
        ink, ink2,
        "scale must not change the resolved colour, only where it lands"
    );
}
