//! E13.0 — a text primitive reaches the screen.
//!
//! The third key under D46: `primitive: "text"`, measured through the `Glyphs` port
//! and drawn through glyphon (E14). A readback asserts known string pixels differ
//! from the background in the placed rect, at two scale factors.

use infinite_presenter::core::{Point, SurfaceRect, TEXT};
use infinite_solutions::editor;
use infinite_solutions::facade::ports::Surface;
use infinite_solutions::facade::{self, encode_space, SpaceRecord};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

/// A label on the canvas, sibling to the genesis nodes.
const LABEL_KEY: &[u8] = &[0x10, 0x00, 0x01, 0x01, 0x00];

const CANVAS: [u8; 3] = [31, 33, 41];

const RUN: &str = "Hi";

/// Canvas-only sample from `tests/pixels.rs` — avoids genesis nodes and the wire.
const IN_CANVAS_ONLY: (u32, u32) = (250, 450);

fn far_from_canvas(px: [u8; 4]) -> bool {
    (0..3).any(|c| (i32::from(px[c]) - i32::from(CANVAS[c])).abs() > 8)
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
    }
}

fn seeded() -> (tempfile::TempDir, facade::Store) {
    let dir = tempfile::TempDir::new().unwrap();
    let store = facade::open(dir.path()).expect("open store");
    editor::seed(|k| store.has(k), |k, v| store.put(k, v));
    store.put(LABEL_KEY, &encode_space(&label_record()));
    store.put_payload(LABEL_KEY, RUN.as_bytes());
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

/// Scan the placed text rect for a pixel that is not the canvas.
fn find_ink(store: &facade::Store, scale: f64) -> [u8; 4] {
    let Some(mut surf) = surface(scale) else {
        panic!("no GPU adapter available");
    };
    store.set_surface(0.0, 0.0, f64::from(WIDTH), f64::from(HEIGHT), scale);
    let placement = store.place_now();
    let placed = placement
        .placed
        .iter()
        .find(|p| p.at.as_bytes() == LABEL_KEY)
        .expect("the label is placed");
    let showing = placed.showing();
    store.draw_with(&mut surf);
    let pixels = surf.read_back().expect("read back");

    let x0 = (showing.min.x * scale).floor().max(0.0) as u32;
    let y0 = (showing.min.y * scale).floor().max(0.0) as u32;
    let x1 = (showing.max.x * scale)
        .ceil()
        .min(f64::from(WIDTH - 1)) as u32;
    let y1 = (showing.max.y * scale)
        .ceil()
        .min(f64::from(HEIGHT - 1)) as u32;

    for y in y0..=y1 {
        for x in x0..=x1 {
            let px = Surface::pixel(&pixels, WIDTH, x, y);
            if far_from_canvas(px) {
                return px;
            }
        }
    }
    panic!(
        "no ink found in text rect ({x0},{y0})-({x1},{y1}) at scale {scale}"
    );
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

    let ink = find_ink(&store, 1.0);
    assert!(
        far_from_canvas(ink),
        "text ink must differ from the canvas, got {ink:?}"
    );

    let background = Surface::pixel(&pixels, WIDTH, IN_CANVAS_ONLY.0, IN_CANVAS_ONLY.1);
    assert!(
        !far_from_canvas(background),
        "off the run is still the canvas, got {background:?}"
    );
    assert_ne!(
        ink, background,
        "text ink and background must differ, or the test proves nothing"
    );

    let ink2 = find_ink(&store, 2.0);
    assert!(
        far_from_canvas(ink2),
        "text ink at scale 2.0 must still differ from the canvas, got {ink2:?}"
    );
}
