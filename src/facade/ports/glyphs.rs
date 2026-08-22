//! [`Glyphs`] — presenter. Text extent and raster.
//!
//! Text measurement stub. A shaping crate waits for a font in the style table
//! (bootstrap plan §9 finding 10).

use infinite_presenter::binding::ports::Glyphs as Port;
use infinite_presenter::core::{Point, Rect};

/// Text measurement. E4 replaces this once a style names a font.
pub struct Glyphs;

impl Glyphs {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl Port for Glyphs {
    fn measure(&self, run: &str, size: f64) -> Rect {
        let width = size * run.chars().count() as f64;
        Rect::new(Point::ORIGIN, Point::new(width, size))
    }
}
