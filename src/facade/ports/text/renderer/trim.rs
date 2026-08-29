use super::TextRenderer;

impl TextRenderer {
    /// Trim the atlas and drop stale cache entries after a frame.
    pub fn trim(&mut self) {
        self.atlas.trim();
        if self.cache.len() > 256 {
            self.cache.clear();
        }
        self.pending.clear();
    }
}
