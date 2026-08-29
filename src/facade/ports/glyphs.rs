//! [`Glyphs`] — presenter. Text extent and raster.
//!
//! E13.0: a bootstrap bitmap font until a style row names a real one (O26). The
//! presenter asks only *how much room*; this file turns a run into ink cells the
//! `Surface` narrows to `f32` and draws.

use infinite_presenter::binding::ports::Glyphs as Port;
use infinite_presenter::core::{Point, Rect};

/// The bootstrap font and raster layout.
pub struct Glyphs;

/// Columns and rows in the bootstrap font.
const COLS: f64 = 5.0;
const ROWS: f64 = 7.0;

impl Glyphs {
    pub(crate) fn new() -> Self {
        Self
    }

    /// Every lit cell in a run, as axis-aligned rectangles in the same coordinates
    /// the placement uses (logical surface units).
    pub fn ink_cells(run: &str, origin: Point, em: f64) -> impl Iterator<Item = Rect> + '_ {
        let cell_h = em / ROWS;
        let cell_w = cell_h * (COLS / ROWS);
        let gap = cell_w * 0.25;
        let advance = COLS * cell_w + gap;
        run.chars().enumerate().flat_map(move |(i, ch)| {
            let base_x = origin.x + i as f64 * advance;
            glyph_bits(ch).into_iter().enumerate().flat_map(move |(row, bits)| {
                (0..5).filter_map(move |col| {
                    if bits & (1 << (4 - col)) == 0 {
                        return None;
                    }
                    Some(Rect::new(
                        Point::new(base_x + col as f64 * cell_w, origin.y + row as f64 * cell_h),
                        Point::new(
                            base_x + (col as f64 + 1.0) * cell_w,
                            origin.y + (row as f64 + 1.0) * cell_h,
                        ),
                    ))
                })
            })
        })
    }
}

impl Port for Glyphs {
    fn measure(&self, run: &str, em: f64) -> Rect {
        if run.is_empty() {
            return Rect::new(Point::ORIGIN, Point::ORIGIN);
        }
        let cell_h = em / ROWS;
        let cell_w = cell_h * (COLS / ROWS);
        let gap = cell_w * 0.25;
        let advance = COLS * cell_w + gap;
        let width = advance * run.chars().count() as f64 - gap;
        Rect::new(Point::ORIGIN, Point::new(width.max(cell_w), em))
    }
}

fn glyph_bits(ch: char) -> [u8; 7] {
    match ch {
        'A' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x00],
        'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
        'C' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
        'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'I' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
        'i' => [0x04, 0x00, 0x04, 0x04, 0x04, 0x04, 0x0E],
        ' ' => [0x00; 7],
        _ => [0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F],
    }
}