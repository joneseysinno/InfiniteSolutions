use crate::facade::ports::text::TextKey;

use super::TextRenderer;

impl TextRenderer {
    /// Shaped ink size of a run at `size` (logical units), weight 400.
    pub fn measure(&mut self, text: &str, size: f32) -> (f32, f32) {
        self.measure_weighted(text, size, 400)
    }

    /// Like [`Self::measure`], with an explicit font weight.
    pub fn measure_weighted(&mut self, text: &str, size: f32, weight: u16) -> (f32, f32) {
        if text.is_empty() {
            return (0.0, 0.0);
        }
        let key = TextKey {
            text: text.to_string(),
            size_milli: (size * 1000.0) as u32,
            weight,
        };
        self.ensure_buffer(&key);
        if let Some(buf) = self.buffer(&key) {
            let mut w = 0.0f32;
            for run in buf.layout_runs() {
                w = w.max(run.line_w);
            }
            return (w.max(size * 0.1), size * 1.35);
        }
        (text.chars().count() as f32 * size * 0.55, size * 1.35)
    }
}
