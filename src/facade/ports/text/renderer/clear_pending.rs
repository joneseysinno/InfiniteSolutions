use super::TextRenderer;

impl TextRenderer {
    /// Drop queued runs without drawing them.
    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }
}
