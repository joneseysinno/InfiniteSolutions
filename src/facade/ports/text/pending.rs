use glyphon::{Color, TextBounds};

use super::TextKey;

#[derive(Clone)]
pub(crate) struct PendingText {
    pub(crate) key: TextKey,
    pub(crate) left: f32,
    pub(crate) top: f32,
    pub(crate) bounds: TextBounds,
    pub(crate) color: Color,
}
