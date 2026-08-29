use glyphon::{Buffer, Resolution, TextArea};

use crate::facade::ports::text::TextKey;

use super::TextRenderer;

impl TextRenderer {
    /// Shape pending runs into the atlas for the next render pass.
    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.viewport.update(
            queue,
            Resolution {
                width: self.width,
                height: self.height,
            },
        );

        let pending = self.pending.clone();
        for item in &pending {
            self.ensure_buffer(&item.key);
        }

        let mut stolen: Vec<(TextKey, Buffer)> = Vec::with_capacity(pending.len());
        for item in &pending {
            stolen.push((item.key.clone(), self.take_buffer(&item.key)));
        }

        let areas: Vec<TextArea> = pending
            .iter()
            .zip(stolen.iter())
            .map(|(item, (_k, buffer))| TextArea {
                buffer,
                left: item.left,
                top: item.top,
                // Resolution + positions are logical; DPI must not multiply glyph size.
                scale: 1.0,
                bounds: item.bounds,
                default_color: item.color,
                custom_glyphs: &[],
            })
            .collect();

        let _ = self.renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            areas,
            &mut self.swash_cache,
        );

        for (key, buffer) in stolen {
            self.insert_buffer(key, buffer);
        }
    }
}
