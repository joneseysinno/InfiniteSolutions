//! [`Glyphs`] — presenter. Text extent via a real font (E14, R-F).
//!
//! Rasterisation lives on [`Surface`](super::Surface) through
//! [`TextRenderer`](super::text::TextRenderer). This port answers only *how much
//! room* a run needs, with the same embedded Inter face the surface draws.

use std::sync::Mutex;

use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Weight};
use infinite_presenter::binding::ports::Glyphs as Port;
use infinite_presenter::core::{Point, Rect};

use super::text::{FONT_BYTES, FONT_FAMILY};

/// The bootstrap measurement face — same bytes the surface rasterises.
pub struct Glyphs {
    font_system: Mutex<FontSystem>,
}

impl Glyphs {
    pub(crate) fn new() -> Self {
        let mut font_system = FontSystem::new();
        font_system.db_mut().load_font_data(FONT_BYTES.to_vec());
        font_system.db_mut().set_sans_serif_family(FONT_FAMILY);
        Self {
            font_system: Mutex::new(font_system),
        }
    }
}

impl Port for Glyphs {
    fn measure(&self, run: &str, em: f64) -> Rect {
        if run.is_empty() {
            return Rect::new(Point::ORIGIN, Point::ORIGIN);
        }
        let size = em as f32;
        let metrics = Metrics::new(size, size * 1.35);
        let mut font_system = self.font_system.lock().expect("glyphs font lock");
        let mut buffer = Buffer::new(&mut font_system, metrics);
        let attrs = Attrs::new()
            .family(Family::SansSerif)
            .weight(Weight(400));
        buffer.set_size(Some(4000.0), Some(metrics.line_height));
        buffer.set_text(run, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut font_system, false);
        let mut width = 0.0f32;
        for layout in buffer.layout_runs() {
            width = width.max(layout.line_w);
        }
        let height = metrics.line_height;
        Rect::new(
            Point::ORIGIN,
            Point::new(f64::from(width.max(size * 0.1)), f64::from(height)),
        )
    }
}
