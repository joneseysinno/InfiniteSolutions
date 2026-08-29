use glyphon::{Attrs, Buffer, Family, Metrics, Shaping, Weight};

use crate::facade::ports::text::TextKey;

use super::TextRenderer;

impl TextRenderer {
    pub(crate) fn make_buffer(&mut self, key: &TextKey) -> Buffer {
        let size = key.size_milli as f32 / 1000.0;
        let metrics = Metrics::new(size, size * 1.35);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);
        let attrs = Attrs::new()
            .family(Family::SansSerif)
            .weight(Weight(key.weight));
        buffer.set_size(Some(4000.0), Some(metrics.line_height));
        buffer.set_text(&key.text, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut self.font_system, false);
        buffer
    }
}
