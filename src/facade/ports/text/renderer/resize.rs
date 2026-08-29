use super::TextRenderer;

impl TextRenderer {
    /// Resize the text viewport in **logical** pixels and set HiDPI scale.
    pub fn resize(&mut self, width: u32, height: u32, scale_factor: f32) {
        self.width = width.max(1);
        self.height = height.max(1);
        self.scale_factor = scale_factor.max(0.01);
    }
}
