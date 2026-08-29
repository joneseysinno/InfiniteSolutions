use super::TextRenderer;

impl TextRenderer {
    /// Draw prepared glyphs into an open render pass.
    pub fn render_into<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
    ) -> Result<(), glyphon::RenderError> {
        self.renderer.render(&self.atlas, &self.viewport, pass)
    }
}
