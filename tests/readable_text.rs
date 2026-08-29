//! E14 — text you can read.
//!
//! A run containing every digit and mixed case renders with **distinct ink per
//! character**: two different characters produce different ink, and no character
//! resolves to the bootstrap hollow-box fallback (finding 23).
//!
//! *Must stop passing:* the old 5×7 bitmap where `1` and `7` shared the same ink.

use infinite_presenter::core::{Point, SurfaceRect, TEXT};
use infinite_solutions::editor;
use infinite_solutions::facade::ports::Surface;
use infinite_solutions::facade::{self, encode_space, SpaceRecord};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

/// One label on the canvas. Rewritten per character so fingerprints need no advance math.
const LABEL_KEY: &[u8] = &[0x10, 0x00, 0x01, 0x01, 0x00];

const CANVAS: [u8; 3] = [31, 33, 41];

/// Digits and mixed case — the E14 green-check corpus.
const CORPUS: &str = "0123456789AaBbCc";

fn far_from_canvas(px: [u8; 4]) -> bool {
    (0..3).any(|c| (i32::from(px[c]) - i32::from(CANVAS[c])).abs() > 8)
}

fn label_record(text: &str) -> SpaceRecord {
    SpaceRecord {
        across: [0.0, 0.0, 0.0],
        down: [0.0, 0.15, 0.0],
        style: "plain".into(),
        detail_override: None,
        hosts_space: false,
        accepts: true,
        // Bottom of the canvas — away from genesis nodes that would dominate a fingerprint.
        origin: [0.05, 0.70],
        primitive: TEXT.into(),
        link: None,
        text: text.into(),
    }
}

fn seeded() -> (tempfile::TempDir, facade::Store) {
    let dir = tempfile::TempDir::new().unwrap();
    let store = facade::open(dir.path()).expect("open store");
    editor::seed(|k| store.has(k), |k, v| store.put(k, v));
    store.put(LABEL_KEY, &encode_space(&label_record("0")));
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

/// Raw RGBA crop from the text origin — different glyphs differ in the bytes.
fn ink_fingerprint(pixels: &[u8], showing: infinite_presenter::core::Rect) -> Vec<u8> {
    let x0 = showing.min.x.floor().max(0.0) as u32;
    let y0 = showing.min.y.floor().max(0.0) as u32;
    let x1 = (showing.min.x + (showing.max.x - showing.min.x).min(80.0))
        .ceil()
        .min(f64::from(WIDTH - 1)) as u32;
    let y1 = (showing.min.y + (showing.max.y - showing.min.y).min(80.0))
        .ceil()
        .min(f64::from(HEIGHT - 1)) as u32;
    let mut out = Vec::new();
    let mut lit = 0usize;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let px = Surface::pixel(pixels, WIDTH, x, y);
            if far_from_canvas(px) {
                lit += 1;
            }
            out.extend_from_slice(&px);
        }
    }
    assert!(
        lit > 0,
        "crop ({x0},{y0})-({x1},{y1}) has no ink — glyph missing or wrong origin"
    );
    out
}

fn draw_char(store: &facade::Store, ch: char) -> Vec<u8> {
    store.put(LABEL_KEY, &encode_space(&label_record(&ch.to_string())));
    let Some(mut surf) = surface() else {
        panic!("no GPU adapter available");
    };
    store.draw_with(&mut surf);
    let pixels = surf.read_back().expect("read back");
    let placement = store.place_now();
    let placed = placement
        .placed
        .iter()
        .find(|p| p.at.as_bytes() == LABEL_KEY)
        .unwrap_or_else(|| panic!("character {ch:?} was not placed"));
    ink_fingerprint(&pixels, placed.showing())
}

#[test]
fn every_digit_and_mixed_case_has_distinct_ink() {
    let (_dir, store) = seeded();
    if surface().is_none() {
        eprintln!("no GPU adapter available; skipping");
        return;
    }

    let mut prints: Vec<(char, Vec<u8>)> = Vec::new();
    for ch in CORPUS.chars() {
        let fp = draw_char(&store, ch);
        prints.push((ch, fp));
    }

    for i in 0..prints.len() {
        for j in (i + 1)..prints.len() {
            assert_ne!(
                prints[i].1, prints[j].1,
                "characters {:?} and {:?} share ink — the bootstrap font defect",
                prints[i].0, prints[j].0
            );
        }
    }

    let one = prints.iter().find(|(c, _)| *c == '1').expect("1 in corpus");
    let seven = prints.iter().find(|(c, _)| *c == '7').expect("7 in corpus");
    assert_ne!(
        one.1, seven.1,
        "digits 1 and 7 must differ — the plan's named must-stop-passing case"
    );
}
